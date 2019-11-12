use crate::file_dialog::FileDialogMode;
use crate::string::CefString;
use crate::string::CefStringList;
use std::mem::ManuallyDrop;
use cef_sys::cef_string_t;
use cef_sys::cef_string_list_t;
use crate::{
    browser::{Browser},
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_browser_t,
    cef_dialog_handler_t,
    cef_file_dialog_callback_t,
    cef_file_dialog_mode_t,
};
use std::os::raw::{c_int};
use parking_lot::Mutex;

ref_counted_ptr!{
    /// Instantiate this structure to handle dialog events.
    ///
    /// The functions of this structure will be called on the browser process UI thread.
    pub struct DialogHandler(*mut cef_dialog_handler_t);
}

ref_counted_ptr!{
    /// Callback structure for asynchronous continuation of file dialog requests.
    pub struct FileDialogCallback(*mut cef_file_dialog_callback_t);
}

impl DialogHandler {
    pub fn new<C: DialogHandlerCallbacks>(callbacks: C) -> DialogHandler {
        unsafe{ DialogHandler::from_ptr_unchecked(DialogHandlerWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

impl FileDialogCallback {
    pub fn new<C: FileDialogCallbacks>(callbacks: C) -> FileDialogCallback {
        unsafe{ FileDialogCallback::from_ptr_unchecked(FileDialogCallbackWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Implement this trait to handle dialog events.
///
/// The functions of this structure will be called on the browser process UI thread.
pub trait DialogHandlerCallbacks: 'static + Send {
    fn on_file_dialog(
        &self,
        browser: Browser,
        mode: FileDialogMode,
        title: Option<&str>,
        default_file_path: &str,
        accept_filters: &[String],
        selected_accept_filter: usize,
        callback: FileDialogCallback,
    ) -> bool {
        false
    }
}

/// Callback trait for asynchronous continuation of file dialog requests.
pub trait FileDialogCallbacks: 'static + Send + Sync {
    /// Continue the file selection. `selected_accept_filter` should be the 0-based
    /// index of the value selected from the accept filters array passed to
    /// cef_dialog_handler_t::OnFileDialog. `file_paths` should be a single value
    /// or a list of values depending on the dialog mode.
    fn cont(
        &self,
        selected_accept_filter: usize,
        file_paths: &[String],
    );
    /// Cancel the file selection.
    fn cancel(&self);
}

struct DialogHandlerWrapper(Mutex<Box<dyn DialogHandlerCallbacks>>);
impl Wrapper for DialogHandlerWrapper {
    type Cef = cef_dialog_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_dialog_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_file_dialog: Some(Self::on_file_dialog),
            },
            self,
        )
    }
}

struct FileDialogCallbackWrapper(Box<dyn FileDialogCallbacks>);
impl Wrapper for FileDialogCallbackWrapper {
    type Cef = cef_file_dialog_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_file_dialog_callback_t {
                base: unsafe { std::mem::zeroed() },
                cont: Some(Self::cont),
                cancel: Some(Self::cancel),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for DialogHandlerWrapper: cef_dialog_handler_t {
        fn on_file_dialog(
            &self,
            browser: Browser: *mut cef_browser_t,
            mode: FileDialogMode: cef_file_dialog_mode_t,
            title: Option<&CefString>: *const cef_string_t,
            default_file_path: &CefString: *const cef_string_t,
            accept_filters: ManuallyDrop<CefStringList>: cef_string_list_t,
            selected_accept_filter: c_int: c_int,
            callback: FileDialogCallback: *mut cef_file_dialog_callback_t
        ) -> c_int {
            self.0.lock().on_file_dialog(
                browser,
                mode,
                title.map(String::from)
                    .as_ref()
                    .map(|t| &**t),
                &String::from(default_file_path),
                &(&*accept_filters).into_iter().map(String::from).collect::<Vec<_>>(),
                selected_accept_filter as usize,
                callback,
            ) as c_int
        }
    }
}

cef_callback_impl!{
    impl for FileDialogCallbackWrapper: cef_file_dialog_callback_t {
        fn cont(
            &self,
            selected_accept_filter: c_int: c_int,
            file_paths: Option<ManuallyDrop<CefStringList>>: cef_string_list_t
        ) {
            if let Some(file_paths) = file_paths {
                self.0.cont(
                    selected_accept_filter as usize,
                    &(&*file_paths).into_iter().map(String::from).collect::<Vec<_>>()
                );
            } else {
                self.0.cancel();
            }
        }
        fn cancel(&self) {
            self.0.cancel();
        }
    }
}
