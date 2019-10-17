use parking_lot::Mutex;
use cef_sys::{
    cef_file_dialog_mode_t, cef_run_file_dialog_callback_t,
    cef_string_list_t,
};
use std::{collections::HashSet, convert::TryFrom};

use crate::{
    string::CefStringList,
    refcounted::{RefCountedPtr, Wrapper},
};

#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum FileDialogModeFlags {
    /// Prompt to overwrite if the user selects an existing file with the Save
    /// dialog.
    OverwritePrompt = cef_file_dialog_mode_t::FILE_DIALOG_OVERWRITEPROMPT_FLAG.0,
    /// Do not display read-only files.
    HideReadOnly = cef_file_dialog_mode_t::FILE_DIALOG_HIDEREADONLY_FLAG.0,
}

/// Supported file dialog modes.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum FileDialogMode {
    /// Requires that the file exists before allowing the user to pick it.
    Open(HashSet<FileDialogModeFlags>),
    /// Like Open, but allows picking multiple files to open.
    OpenMultiple(HashSet<FileDialogModeFlags>),
    /// Like Open, but selects a folder to open.
    OpenFolder(HashSet<FileDialogModeFlags>),
    /// Allows picking a nonexistent file, and prompts to overwrite if the file
    /// already exists.
    Save(HashSet<FileDialogModeFlags>),
}

impl TryFrom<cef_file_dialog_mode_t> for FileDialogMode {
    type Error = ();

    fn try_from(value: cef_file_dialog_mode_t) -> Result<Self, Self::Error> {
        let base = value & cef_file_dialog_mode_t::FILE_DIALOG_TYPE_MASK;
        let mut flags = HashSet::new();
        if (value & cef_file_dialog_mode_t::FILE_DIALOG_OVERWRITEPROMPT_FLAG).0 != 0 {
            flags.insert(FileDialogModeFlags::OverwritePrompt);
        }
        if (value & cef_file_dialog_mode_t::FILE_DIALOG_HIDEREADONLY_FLAG).0 != 0 {
            flags.insert(FileDialogModeFlags::HideReadOnly);
        }
        match base {
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_OPEN => Ok(Self::Open(flags)),
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_OPEN_MULTIPLE => {
                Ok(Self::OpenMultiple(flags))
            }
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_OPEN_FOLDER => {
                Ok(Self::OpenFolder(flags))
            }
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_SAVE => Ok(Self::Save(flags)),
            _ => Err(()),
        }
    }
}

impl Into<cef_file_dialog_mode_t> for FileDialogMode {
    fn into(self) -> cef_file_dialog_mode_t {
        let result;
        let flags = match self {
            Self::Open(flags) => {
                result = cef_file_dialog_mode_t::FILE_DIALOG_OPEN.0;
                flags
            }
            Self::OpenMultiple(flags) => {
                result = cef_file_dialog_mode_t::FILE_DIALOG_OPEN_MULTIPLE.0;
                flags
            }
            Self::OpenFolder(flags) => {
                result = cef_file_dialog_mode_t::FILE_DIALOG_OPEN_FOLDER.0;
                flags
            }
            Self::Save(flags) => {
                result = cef_file_dialog_mode_t::FILE_DIALOG_SAVE.0;
                flags
            }
        };
        cef_file_dialog_mode_t(flags
            .into_iter()
            .fold(result, |result, flag| result | flag as i32))
    }
}

ref_counted_ptr!{
    pub(crate) struct RunFileDialogCallback(*mut cef_run_file_dialog_callback_t);
}

pub(crate) struct RunFileDialogCallbackWrapper {
    callback: Mutex<Option<Box<dyn Send + FnOnce(usize, Option<Vec<String>>)>>>
}

impl Wrapper for RunFileDialogCallbackWrapper {
    type Cef = cef_run_file_dialog_callback_t;
    type Inner = Mutex<Option<Box<dyn Send + FnOnce(usize, Option<Vec<String>>)>>>;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_run_file_dialog_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_file_dialog_dismissed: Some(Self::file_dialog_dismissed),
            },
            self,
        )
    }
}

impl RunFileDialogCallbackWrapper {
    pub(crate) fn new<F>(callback: F) -> RunFileDialogCallbackWrapper
    where
        F: 'static + Send + FnOnce(usize, Option<Vec<String>>),
    {
        RunFileDialogCallbackWrapper {
            callback: Mutex::new(Some(Box::new(callback)))
        }
    }
}

cef_callback_impl!{
    impl for RunFileDialogCallbackWrapper: cef_run_file_dialog_callback_t {
        fn file_dialog_dismissed(
            &self,
            selected_accept_filter: std::os::raw::c_int: std::os::raw::c_int,
            file_paths: Option<CefStringList>: cef_string_list_t,
        ) {
            // file_dialog_dismissed consumes self
            if let Some(callback) = self.callback.lock().take() {
                // we can only call FnOnce once, so it has to be consumed here
                callback(
                    selected_accept_filter as usize,
                    file_paths.map(Vec::from),
                );
            }
        }
    }
}
