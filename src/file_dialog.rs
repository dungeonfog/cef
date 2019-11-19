use cef_sys::{cef_file_dialog_mode_t, cef_run_file_dialog_callback_t, cef_string_list_t};
use parking_lot::Mutex;
use std::{
    convert::TryFrom,
    mem::ManuallyDrop,
};
use crate::{
    refcounted::{RefCountedPtr, Wrapper},
    string::CefStringList,
};
use bitflags::bitflags;

bitflags!{
    pub struct FileDialogModeFlags: crate::CEnumType {
        const OVERWRITE_PROMPT = cef_file_dialog_mode_t::FILE_DIALOG_OVERWRITEPROMPT_FLAG.0;
        const HIDE_READ_ONLY = cef_file_dialog_mode_t::FILE_DIALOG_HIDEREADONLY_FLAG.0;
    }
}

/// Supported file dialog modes.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum FileDialogMode {
    /// Requires that the file exists before allowing the user to pick it.
    Open(FileDialogModeFlags),
    /// Like Open, but allows picking multiple files to open.
    OpenMultiple(FileDialogModeFlags),
    /// Like Open, but selects a folder to open.
    OpenFolder(FileDialogModeFlags),
    /// Allows picking a nonexistent file, and prompts to overwrite if the file
    /// already exists.
    Save(FileDialogModeFlags),
}

impl TryFrom<cef_file_dialog_mode_t> for FileDialogMode {
    type Error = ();

    fn try_from(value: cef_file_dialog_mode_t) -> Result<Self, Self::Error> {
        let base = value & cef_file_dialog_mode_t::FILE_DIALOG_TYPE_MASK;
        let mut flags = FileDialogModeFlags::empty();
        if (value & cef_file_dialog_mode_t::FILE_DIALOG_OVERWRITEPROMPT_FLAG).0 != 0 {
            flags.insert(FileDialogModeFlags::OVERWRITE_PROMPT);
        }
        if (value & cef_file_dialog_mode_t::FILE_DIALOG_HIDEREADONLY_FLAG).0 != 0 {
            flags.insert(FileDialogModeFlags::HIDE_READ_ONLY);
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
        let flags = match self {
            Self::Open(flags) => {
                cef_file_dialog_mode_t::FILE_DIALOG_OPEN.0 | flags.bits()
            }
            Self::OpenMultiple(flags) => {
                cef_file_dialog_mode_t::FILE_DIALOG_OPEN_MULTIPLE.0 | flags.bits()
            }
            Self::OpenFolder(flags) => {
                cef_file_dialog_mode_t::FILE_DIALOG_OPEN_FOLDER.0 | flags.bits()
            }
            Self::Save(flags) => {
                cef_file_dialog_mode_t::FILE_DIALOG_SAVE.0 | flags.bits()
            }
        };
        cef_file_dialog_mode_t(
            flags
        )
    }
}

ref_counted_ptr! {
    pub(crate) struct RunFileDialogCallback(*mut cef_run_file_dialog_callback_t);
}

pub(crate) struct RunFileDialogCallbackWrapper {
    callback: Mutex<Option<Box<dyn Send + FnOnce(usize, Option<Vec<String>>)>>>,
}

impl Wrapper for RunFileDialogCallbackWrapper {
    type Cef = cef_run_file_dialog_callback_t;
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
            callback: Mutex::new(Some(Box::new(callback))),
        }
    }
}

cef_callback_impl! {
    impl for RunFileDialogCallbackWrapper: cef_run_file_dialog_callback_t {
        fn file_dialog_dismissed(
            &self,
            selected_accept_filter: std::os::raw::c_int: std::os::raw::c_int,
            file_paths: Option<ManuallyDrop<CefStringList>>: cef_string_list_t,
        ) {
            // file_dialog_dismissed consumes self
            if let Some(callback) = self.callback.lock().take() {
                // we can only call FnOnce once, so it has to be consumed here
                callback(
                    selected_accept_filter as usize,
                    file_paths.map(|p| (&*p).into_iter().map(String::from).collect()),
                );
            }
        }
    }
}
