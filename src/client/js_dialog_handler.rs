use crate::string::CefString;
use cef_sys::cef_string_t;
use crate::{
    browser::{Browser},
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_browser_t,
    cef_jsdialog_handler_t,
    cef_jsdialog_callback_t,
    cef_jsdialog_type_t,
};
use std::os::raw::{c_int};
use parking_lot::Mutex;

ref_counted_ptr!{
    /// Instantiate this structure to handle events related to JavaScript dialogs. The
    /// functions of this structure will be called on the UI thread.
    pub struct JsDialogHandler(*mut cef_jsdialog_handler_t);
}

ref_counted_ptr!{
    /// Callback struct used for asynchronous continuation of JavaScript dialog
    /// requests.
    pub struct JsDialogCallback(*mut cef_jsdialog_callback_t);
}

/// Supported JavaScript dialog types.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum JsDialogType {
    Alert = cef_jsdialog_type_t::JSDIALOGTYPE_ALERT as isize,
    Confirm = cef_jsdialog_type_t::JSDIALOGTYPE_CONFIRM as isize,
    Prompt = cef_jsdialog_type_t::JSDIALOGTYPE_PROMPT as isize,
}

impl JsDialogType {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

impl JsDialogHandler {
    pub fn new<C: JsDialogHandlerCallbacks>(callbacks: C) -> JsDialogHandler {
        unsafe{ JsDialogHandler::from_ptr_unchecked(JsDialogHandlerWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

impl JsDialogCallback {
    // Continue the JS dialog request. Set `success` to `true` if the OK button
    // was pressed. The `user_input` value should be specified for prompt dialogs.
    pub fn cont(
        &self,
        success: bool,
        user_input: &str,
    ) {
        unsafe{
            self.0.cont.unwrap()(
                self.as_ptr(),
                success as c_int,
                CefString::new(user_input).as_ptr(),
            )
        }
    }
}

/// Implement this trait to handle events related to JavaScript dialogs. The
/// functions of this structure will be called on the UI thread.
pub trait JsDialogHandlerCallbacks: 'static + Send {
    // Called to run a JavaScript dialog. If `origin_url` is non-NULL it can be
    // passed to the CefFormatUrlForSecurityDisplay function to retrieve a secure
    // and user-friendly display string. The `default_prompt_text` value will be
    // specified for prompt dialogs only. Set `suppress_message` to `true` and
    // return `false` to suppress the message (suppressing messages is
    // preferable to immediately executing the callback as this is used to detect
    // presumably malicious behavior like spamming alert messages in
    // onbeforeunload). Set `suppress_message` to `false` and return `false`
    // to use the default implementation (the default implementation will show one
    // modal dialog at a time and suppress any additional dialog requests until
    // the displayed dialog is dismissed). Return `true` if the application will
    // use a custom dialog or if the callback has been executed immediately.
    // Custom dialogs may be either modal or modeless. If a custom dialog is used
    // the application must execute `callback` once the custom dialog is
    // dismissed.
    fn on_js_dialog(
        &mut self,
        browser: Browser,
        origin_url: &str,
        dialog_type: JsDialogType,
        message_text: &str,
        default_prompt_text: &str,
        callback: JsDialogCallback,
        suppress_message: &mut bool,
    ) -> bool {
        false
    }
    // Called to run a dialog asking the user if they want to leave a page. Return
    // `false` to use the default dialog implementation. Return `true` if the
    // application will use a custom dialog or if the callback has been executed
    // immediately. Custom dialogs may be either modal or modeless. If a custom
    // dialog is used the application must execute `callback` once the custom
    // dialog is dismissed.
    fn on_before_unload_dialog(
        &mut self,
        browser: Browser,
        message_text: &str,
        is_reload: bool,
        callback: JsDialogCallback,
    ) -> bool {
        false
    }
    // Called to cancel any pending dialogs and reset any saved dialog state. Will
    // be called due to events like page navigation irregardless of whether any
    // dialogs are currently pending.
    fn on_reset_dialog_state(
        &mut self,
        browser: Browser,
    ) {
    }
    // Called when the default implementation dialog is closed.
    fn on_dialog_closed(
        &mut self,
        browser: Browser,
    ) {
    }
}

struct JsDialogHandlerWrapper(Mutex<Box<dyn JsDialogHandlerCallbacks>>);
impl Wrapper for JsDialogHandlerWrapper {
    type Cef = cef_jsdialog_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_jsdialog_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_jsdialog: Some(Self::on_jsdialog),
                on_before_unload_dialog: Some(Self::on_before_unload_dialog),
                on_reset_dialog_state: Some(Self::on_reset_dialog_state),
                on_dialog_closed: Some(Self::on_dialog_closed),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for JsDialogHandlerWrapper: cef_jsdialog_handler_t {
        fn on_jsdialog(
            &self,
            browser: Browser: *mut cef_browser_t,
            origin_url: &CefString: *const cef_string_t,
            dialog_type: JsDialogType: cef_jsdialog_type_t::Type,
            message_text: &CefString: *const cef_string_t,
            default_prompt_text: &CefString: *const cef_string_t,
            callback: JsDialogCallback: *mut cef_jsdialog_callback_t,
            suppress_message: &mut c_int: *mut c_int
        ) -> c_int {
            let mut suppress_message_rs = *suppress_message != 0;
            let ret =self.0.lock().on_js_dialog(
                browser,
                &String::from(origin_url),
                dialog_type,
                &String::from(message_text),
                &String::from(default_prompt_text),
                callback,
                &mut suppress_message_rs
            ) as c_int;
            *suppress_message = suppress_message_rs as c_int;
            ret
        }
        fn on_before_unload_dialog(
            &self,
            browser: Browser: *mut cef_browser_t,
            message_text: &CefString: *const cef_string_t,
            is_reload: c_int: c_int,
            callback: JsDialogCallback: *mut cef_jsdialog_callback_t
        ) -> c_int {
            self.0.lock().on_before_unload_dialog(
                browser,
                &String::from(message_text),
                is_reload != 0,
                callback,
            ) as c_int
        }
        fn on_reset_dialog_state(
            &self,
            browser: Browser: *mut cef_browser_t
        ) {
            self.0.lock().on_reset_dialog_state(browser);
        }
        fn on_dialog_closed(
            &self,
            browser: Browser: *mut cef_browser_t
        ) {
            self.0.lock().on_dialog_closed(browser);
        }
    }
}
