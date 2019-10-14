use cef_sys::{cef_frame_t, cef_string_userfree_utf16_free};

use crate::{
    browser::Browser,
    dom::{DOMVisitor, DOMVisitorWrapper},
    request::Request,
    string::{CefString, StringVisitor, StringVisitorWrapper},
    url_request::{URLRequest, URLRequestClient, URLRequestClientWrapper},
    v8context::V8Context,
};

ref_counted_ptr! {
    /// Structure used to represent a frame in the browser window. When used in the
    /// browser process the functions of this structure may be called on any thread
    /// unless otherwise indicated in the comments. When used in the render process
    /// the functions of this structure may only be called on the main thread.
    pub struct Frame(*mut cef_frame_t);
}

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    /// True if this object is currently attached to a valid frame.
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .and_then(|is_valid| Some(unsafe { is_valid(self.0.as_ptr()) } != 0))
            .unwrap_or(false)
    }
    /// Execute undo in this frame.
    pub fn undo(&mut self) {
        if let Some(undo) = self.0.undo {
            unsafe {
                undo(self.0.as_ptr());
            }
        }
    }
    /// Execute redo in this frame.
    pub fn redo(&mut self) {
        if let Some(redo) = self.0.redo {
            unsafe {
                redo(self.0.as_ptr());
            }
        }
    }
    /// Execute cut in this frame.
    pub fn cut(&mut self) {
        if let Some(cut) = self.0.cut {
            unsafe {
                cut(self.0.as_ptr());
            }
        }
    }
    /// Execute copy in this frame.
    pub fn copy(&mut self) {
        if let Some(copy) = self.0.copy {
            unsafe {
                copy(self.0.as_ptr());
            }
        }
    }
    /// Execute paste in this frame.
    pub fn paste(&mut self) {
        if let Some(paste) = self.0.paste {
            unsafe {
                paste(self.0.as_ptr());
            }
        }
    }
    /// Execute delete in this frame.
    pub fn del(&mut self) {
        if let Some(del) = self.0.del {
            unsafe {
                del(self.0.as_ptr());
            }
        }
    }
    /// Execute select all in this frame.
    pub fn select_all(&mut self) {
        if let Some(select_all) = self.0.select_all {
            unsafe {
                select_all(self.0.as_ptr());
            }
        }
    }
    /// Save this frame's HTML source to a temporary file and open it in the
    /// default text viewing application. This function can only be called from the
    /// browser process.
    pub fn view_source(&self) {
        if let Some(view_source) = self.0.view_source {
            unsafe {
                view_source(self.0.as_ptr());
            }
        }
    }
    /// Retrieve this frame's HTML source as a string sent to the specified
    /// visitor.
    pub fn get_source(&self, visitor: Box<dyn StringVisitor>) {
        if let Some(get_source) = self.0.get_source {
            let visitor = StringVisitorWrapper::wrap(visitor);
            unsafe {
                get_source(self.0.as_ptr(), visitor);
            }
        }
    }
    /// Retrieve this frame's display text as a string sent to the specified
    /// visitor.
    pub fn get_text(&self, visitor: Box<dyn StringVisitor>) {
        if let Some(get_text) = self.0.get_text {
            let visitor = StringVisitorWrapper::wrap(visitor);
            unsafe {
                get_text(self.0.as_ptr(), visitor);
            }
        }
    }
    /// Load the request represented by the |request| object.
    pub fn load_request(&mut self, request: &Request) {
        if let Some(load_request) = self.0.load_request {
            unsafe {
                load_request(self.0.as_ptr(), request.as_ptr());
            }
        }
    }
    /// Load the specified `url`.
    pub fn load_url(&mut self, url: &str) {
        if let Some(load_url) = self.0.load_url {
            unsafe {
                load_url(self.0.as_ptr(), CefString::new(url).as_ptr());
            }
        }
    }
    /// Load the contents of `string_val` with the specified dummy `url`. `url`
    /// should have a standard scheme (for example, http scheme) or behaviors like
    /// link clicks and web security restrictions may not behave as expected.
    pub fn load_string(&mut self, string_val: &str, url: &str) {
        if let Some(load_string) = self.0.load_string {
            unsafe {
                load_string(
                    self.0.as_ptr(),
                    CefString::new(string_val).as_ptr(),
                    CefString::new(url).as_ptr(),
                );
            }
        }
    }
    /// Execute a string of JavaScript code in this frame. The `script_url`
    /// parameter is the URL where the script in question can be found, if any. The
    /// renderer may request this URL to show the developer the source of the
    /// error.  The `start_line` parameter is the base line number to use for error
    /// reporting.
    pub fn execute_java_script(&mut self, code: &str, script_url: &str, start_line: i32) {
        if let Some(execute_java_script) = self.0.execute_java_script {
            unsafe {
                execute_java_script(
                    self.0.as_ptr(),
                    CefString::new(code).as_ptr(),
                    CefString::new(script_url).as_ptr(),
                    start_line,
                );
            }
        }
    }
    /// Returns true if this is the main (top-level) frame.
    pub fn is_main(&self) -> bool {
        if let Some(is_main) = self.0.is_main {
            unsafe { is_main(self.0.as_ptr()) != 0 }
        } else {
            false
        }
    }
    /// Returns true if this is the focused frame.
    pub fn is_focused(&self) -> bool {
        if let Some(is_focused) = self.0.is_focused {
            unsafe { is_focused(self.0.as_ptr()) != 0 }
        } else {
            false
        }
    }
    /// Returns the name for this frame. If the frame has an assigned name (for
    /// example, set via the iframe "name" attribute) then that value will be
    /// returned. Otherwise a unique name will be constructed based on the frame
    /// parent hierarchy. The main (top-level) frame will always have an None name
    /// value.
    pub fn get_name(&self) -> Option<String> {
        if let Some(get_name) = self.0.get_name {
            let name = unsafe { get_name(self.0.as_ptr()) };
            let result = unsafe { CefString::from_ptr(name) }.map(String::from);
            if result.is_some() {
                unsafe {
                    cef_string_userfree_utf16_free(name);
                }
            }
            result
        } else {
            None
        }
    }
    /// Returns the globally unique identifier for this frame or None if the
    /// underlying frame does not yet exist.
    pub fn get_identifier(&self) -> Option<i64> {
        if let Some(get_identifier) = self.0.get_identifier {
            let id = unsafe { get_identifier(self.0.as_ptr()) };
            if id < 0 {
                None
            } else {
                Some(id)
            }
        } else {
            None
        }
    }
    /// Returns the parent of this frame or None if this is the main (top-level)
    /// frame.
    pub fn get_parent(&self) -> Option<Frame> {
        if let Some(get_parent) = self.0.get_parent {
            unsafe { Frame::from_ptr(get_parent(self.0.as_ptr())) }
        } else {
            None
        }
    }
    /// Returns the URL currently loaded in this frame.
    pub fn get_url(&self) -> String {
        self.0.get_url
            .and_then(|get_url| unsafe{ get_url(self.as_ptr()).as_mut() })
            .map(|url| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(url));
                cef_string_userfree_utf16_free(url);
                s
            })
            .unwrap_or_default()
    }
    /// Returns the browser that this frame belongs to.
    pub fn get_browser(&self) -> Browser {
        let browser = unsafe { self.0.get_browser.unwrap()(self.0.as_ptr()) };
        unsafe { Browser::from_ptr(browser).expect("CEF: Frame without a browser!") }
    }
    /// Get the V8 context associated with the frame. This function can only be
    /// called from the render process.
    pub fn get_v8context(&self) -> V8Context {
        let context = unsafe { self.0.get_v8context.unwrap()(self.0.as_ptr()) };
        unsafe { V8Context::from_ptr(context).expect("CEF: Frame without a V8 context!") }
    }
    /// Visit the DOM document. This function can only be called from the render
    /// process.
    pub fn visit_dom(&self, visitor: Box<dyn DOMVisitor>) {
        if let Some(visit_dom) = self.0.visit_dom {
            let visitor = DOMVisitorWrapper::wrap(visitor);
            unsafe { visit_dom(self.0.as_ptr(), visitor) };
        }
    }
    /// Create a new URL request that will be treated as originating from this
    /// frame and the associated browser. This request may be intercepted by the
    /// client via [ResourceRequestHandler] or [SchemeHandlerFactory].
    /// Use [URLRequest::new] instead if you do not want the request to have
    /// this association, in which case it may be handled differently (see
    /// documentation on that function). Requests may originate from both the
    /// browser process and the render process.
    ///
    /// For requests originating from the browser process:
    ///   - POST data may only contain a single element of type [PostDataElementType::File] or
    ///     [PostDataElementType::Bytes].
    /// For requests originating from the render process:
    ///   - POST data may only contain a single element of type [PostDataElementType::Bytes].
    ///   - If the response contains Content-Disposition or Mime-Type header values
    ///     that would not normally be rendered then the response may receive
    ///     special handling inside the browser (for example, via the file download
    ///     code path instead of the URL request code path).
    ///
    /// The `request` object will be marked as read-only after calling this
    /// function.
    pub fn create_urlrequest(
        &self,
        request: &mut Request,
        client: Box<dyn URLRequestClient>,
    ) -> URLRequest {
        unsafe {
            let urlrequest = self.0.create_urlrequest.unwrap()(
                self.0.as_ptr(),
                request.as_ptr(),
                URLRequestClientWrapper::wrap(client),
            );
            URLRequest::from_ptr_unchecked(urlrequest)
        }
    }
}
