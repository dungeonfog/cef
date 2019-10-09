use cef_sys::{
    cef_base_ref_counted_t, cef_file_dialog_mode_t, cef_run_file_dialog_callback_t,
    cef_string_list_t,
};
use std::{collections::HashSet, convert::TryFrom};

use crate::{
    refcounted::{RefCounted, RefCounter},
    string::from_string_list,
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

impl TryFrom<i32> for FileDialogMode {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let base = value & cef_file_dialog_mode_t::FILE_DIALOG_TYPE_MASK.0;
        let mut flags = HashSet::new();
        if value & cef_file_dialog_mode_t::FILE_DIALOG_OVERWRITEPROMPT_FLAG.0 != 0 {
            flags.insert(FileDialogModeFlags::OverwritePrompt);
        }
        if value & cef_file_dialog_mode_t::FILE_DIALOG_HIDEREADONLY_FLAG.0 != 0 {
            flags.insert(FileDialogModeFlags::HideReadOnly);
        }
        match base {
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_OPEN.0 => Ok(Self::Open(flags)),
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_OPEN_MULTIPLE.0 => {
                Ok(Self::OpenMultiple(flags))
            }
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_OPEN_FOLDER.0 => {
                Ok(Self::OpenFolder(flags))
            }
            x if x == cef_file_dialog_mode_t::FILE_DIALOG_SAVE.0 => Ok(Self::Save(flags)),
            _ => Err(()),
        }
    }
}

impl Into<i32> for FileDialogMode {
    fn into(self) -> i32 {
        let mut result = 0;
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
        flags
            .into_iter()
            .fold(result, |result, flag| result | flag as i32)
    }
}

pub(crate) struct RunFileDialogCallbackWrapper(*mut cef_run_file_dialog_callback_t);

impl RefCounter for cef_run_file_dialog_callback_t {
    type Wrapper = Option<Box<dyn FnOnce(usize, Option<Vec<String>>)>>;
    fn set_base(&mut self, base: cef_base_ref_counted_t) {
        self.base = base;
    }
}

impl RunFileDialogCallbackWrapper {
    pub(crate) fn new<F>(callback: F) -> *mut cef_run_file_dialog_callback_t
    where
        F: 'static + FnOnce(usize, Option<Vec<String>>),
    {
        let rc = RefCounted::new(
            cef_run_file_dialog_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_file_dialog_dismissed: Some(Self::file_dialog_dismissed),
            },
            Some(Box::new(callback)),
        );
        unsafe { rc.as_mut() }.unwrap().get_cef()
    }

    extern "C" fn file_dialog_dismissed(
        self_: *mut cef_run_file_dialog_callback_t,
        selected_accept_filter: ::std::os::raw::c_int,
        file_paths: cef_string_list_t,
    ) {
        let mut this = unsafe { RefCounted::<cef_run_file_dialog_callback_t>::make_temp(self_) };
        if let Some(callback) = this.take() {
            // we can only call FnOnce once, so it has to be consumed here
            callback(
                selected_accept_filter as usize,
                if file_paths.is_null() {
                    None
                } else {
                    Some(from_string_list(file_paths))
                },
            );
        }
        // no longer needed
        RefCounted::<cef_run_file_dialog_callback_t>::release(this.get_cef() as *mut _);
    }
}
