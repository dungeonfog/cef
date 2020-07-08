use cef_sys::{cef_dev_tools_message_observer_t, cef_browser_t, cef_string_t};
use crate::{
    browser::Browser,
    send_protector::SendProtectorMut, refcounted::{Wrapper, RefCountedPtr}, string::CefString,
};
use std::{os::raw::c_int, ffi::c_void, slice};

ref_counted_ptr!{
    /// Callback structure for cef_browser_host_t::AddDevToolsMessageObserver. The
    /// functions of this structure will be called on the browser process UI thread.
    pub struct DevToolsMessageObserver(*mut cef_dev_tools_message_observer_t);
}

pub trait DevToolsMessageObserverCallbacks: 'static + Send {
    // Method that will be called on receipt of a DevTools protocol message.
    // |browser| is the originating browser instance. |message| is a UTF8-encoded
    // JSON dictionary representing either a function result or an event.
    // |message| is only valid for the scope of this callback and should be copied
    // if necessary. Return true (1) if the message was handled or false (0) if
    // the message should be further processed and passed to the
    // OnDevToolsMethodResult or OnDevToolsEvent functions as appropriate.
    //
    // Method result dictionaries include an "id" (int) value that identifies the
    // orginating function call sent from cef_browser_host_t::SendDevToolsMessage,
    // and optionally either a "result" (dictionary) or "error" (dictionary)
    // value. The "error" dictionary will contain "code" (int) and "message"
    // (string) values. Event dictionaries include a "function" (string) value and
    // optionally a "params" (dictionary) value. See the DevTools protocol
    // documentation at https://chromedevtools.github.io/devtools-protocol/ for
    // details of supported function calls and the expected "result" or "params"
    // dictionary contents. JSON dictionaries can be parsed using the CefParseJSON
    // function if desired, however be aware of performance considerations when
    // parsing large messages (some of which may exceed 1MB in size).
    fn on_dev_tools_message(
        &mut self,
        browser: Browser,
        message: &[u8],
    ) -> bool {
        false
    }
    // Method that will be called after attempted execution of a DevTools protocol
    // function. |browser| is the originating browser instance. |message_id| is
    // the "id" value that identifies the originating function call message. If
    // the function succeeded |success| will be true (1) and |result| will be the
    // UTF8-encoded JSON "result" dictionary value (which may be NULL). If the
    // function failed |success| will be false (0) and |result| will be the
    // UTF8-encoded JSON "error" dictionary value. |result| is only valid for the
    // scope of this callback and should be copied if necessary. See the
    // OnDevToolsMessage documentation for additional details on |result|
    // contents.
    fn on_dev_tools_method_result(
        &mut self,
        browser: Browser,
        message_id: i32,
        success: bool,
        result: &[u8],
    ) {

    }
    // Method that will be called on receipt of a DevTools protocol event.
    // |browser| is the originating browser instance. |function| is the "function"
    // value. |params| is the UTF8-encoded JSON "params" dictionary value (which
    // may be NULL). |params| is only valid for the scope of this callback and
    // should be copied if necessary. See the OnDevToolsMessage documentation for
    // additional details on |params| contents.
    fn on_dev_tools_event(
        &mut self,
        browser: Browser,
        method: &str,
        params: &[u8],
    ) {

    }
    // Method that will be called when the DevTools agent has attached. |browser|
    // is the originating browser instance. This will generally occur in response
    // to the first message sent while the agent is detached.
    fn on_dev_tools_agent_attached(
        &mut self,
        browser: Browser,
    ) {

    }
    // Method that will be called when the DevTools agent has detached. |browser|
    // is the originating browser instance. Any function results that were pending
    // before the agent became detached will not be delivered, and any active
    // event subscriptions will be canceled.
    fn on_dev_tools_agent_detached(
        &mut self,
        browser: Browser,
    ) {

    }
}

struct DevToolsMessageObserverWrapper(SendProtectorMut<Box<dyn DevToolsMessageObserverCallbacks>>);

impl DevToolsMessageObserver {
    fn new(callbacks: impl DevToolsMessageObserverCallbacks) -> DevToolsMessageObserver {
        unsafe{ DevToolsMessageObserver::from_ptr_unchecked(DevToolsMessageObserverWrapper(SendProtectorMut::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

impl Wrapper for DevToolsMessageObserverWrapper {
    type Cef = cef_dev_tools_message_observer_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_dev_tools_message_observer_t {
                base: unsafe { std::mem::zeroed() },
                on_dev_tools_message: Some(Self::on_dev_tools_message),
                on_dev_tools_method_result: Some(Self::on_dev_tools_method_result),
                on_dev_tools_event: Some(Self::on_dev_tools_event),
                on_dev_tools_agent_attached: Some(Self::on_dev_tools_agent_attached),
                on_dev_tools_agent_detached: Some(Self::on_dev_tools_agent_detached),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for DevToolsMessageObserverWrapper: cef_dev_tools_message_observer_t {
        fn on_dev_tools_message(
            &self,
            browser: Browser: *mut cef_browser_t,
            message: *const c_void: *const c_void,
            message_size: usize: usize,
        ) -> c_int {
            let message = unsafe{ slice::from_raw_parts(message as *const u8, message_size) };
            unsafe {
                self.0.get_mut().on_dev_tools_message(browser, message) as _
            }
        }
        fn on_dev_tools_method_result(
            &self,
            browser: Browser: *mut cef_browser_t,
            message_id: i32: c_int,
            success: bool: c_int,
            result: *const c_void: *const c_void,
            result_size: usize: usize,
        ) {
            let result = unsafe{ slice::from_raw_parts(result as *const u8, result_size) };
            unsafe {
                self.0.get_mut().on_dev_tools_method_result(
                    browser,
                    message_id,
                    success,
                    result,
                )
            }
        }
        fn on_dev_tools_event(
            &self,
            browser: Browser: *mut cef_browser_t,
            method: &CefString: *const cef_string_t,
            params: *const c_void: *const c_void,
            params_size: usize: usize,
        ) {
            let params = unsafe{ slice::from_raw_parts(params as *const u8, params_size) };
            unsafe {
                self.0.get_mut().on_dev_tools_event(
                    browser,
                    &*String::from(method),
                    params
                )
            }
        }
        fn on_dev_tools_agent_attached(
            &self,
            browser: Browser: *mut cef_browser_t,
        ) {
            unsafe {
                self.0.get_mut().on_dev_tools_agent_attached(
                    browser,
                )
            }
        }
        fn on_dev_tools_agent_detached(
            &self,
            browser: Browser: *mut cef_browser_t,
        ) {
            unsafe {
                self.0.get_mut().on_dev_tools_agent_detached(
                    browser,
                )
            }
        }
    }
}
