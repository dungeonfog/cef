use cef_sys::{cef_frame_t, cef_string_userfree_utf16_free};

use crate::{
    browser::Browser,
    dom::{DOMVisitor, DOMVisitorWrapper},
    request::Request,
    string::{CefString, StringVisitor, StringVisitorWrapper},
    urlrequest::{URLRequest, URLRequestClient, URLRequestClientWrapper},
    v8context::V8Context,
};

/// Structure used to represent a frame in the browser window. When used in the
/// browser process the functions of this structure may be called on any thread
/// unless otherwise indicated in the comments. When used in the render process
/// the functions of this structure may only be called on the main thread.
pub struct Frame(*mut cef_frame_t);

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

#[doc(hidden)]
impl std::convert::AsRef<cef_frame_t> for Frame {
    fn as_ref(&self) -> &cef_frame_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

#[doc(hidden)]
impl From<*mut cef_frame_t> for Frame {
    fn from(frame: *mut cef_frame_t) -> Self {
        unsafe {
            ((*frame).base.add_ref.unwrap())(&mut (*frame).base);
        }
        Self(frame)
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            (self.as_ref().base.release.unwrap())(&mut (*self.0).base);
        }
    }
}

impl Frame {
    /// True if this object is currently attached to a valid frame.
    pub fn is_valid(&self) -> bool {
        self.as_ref()
            .is_valid
            .and_then(|is_valid| Some(unsafe { is_valid(self.0) } != 0))
            .unwrap_or(false)
    }
    /// Execute undo in this frame.
    pub fn undo(&mut self) {
        if let Some(undo) = self.as_ref().undo {
            unsafe {
                undo(self.0);
            }
        }
    }
    /// Execute redo in this frame.
    pub fn redo(&mut self) {
        if let Some(redo) = self.as_ref().redo {
            unsafe {
                redo(self.0);
            }
        }
    }
    /// Execute cut in this frame.
    pub fn cut(&mut self) {
        if let Some(cut) = self.as_ref().cut {
            unsafe {
                cut(self.0);
            }
        }
    }
    /// Execute copy in this frame.
    pub fn copy(&mut self) {
        if let Some(copy) = self.as_ref().copy {
            unsafe {
                copy(self.0);
            }
        }
    }
    /// Execute paste in this frame.
    pub fn paste(&mut self) {
        if let Some(paste) = self.as_ref().paste {
            unsafe {
                paste(self.0);
            }
        }
    }
    /// Execute delete in this frame.
    pub fn del(&mut self) {
        if let Some(del) = self.as_ref().del {
            unsafe {
                del(self.0);
            }
        }
    }
    /// Execute select all in this frame.
    pub fn select_all(&mut self) {
        if let Some(select_all) = self.as_ref().select_all {
            unsafe {
                select_all(self.0);
            }
        }
    }
    /// Save this frame's HTML source to a temporary file and open it in the
    /// default text viewing application. This function can only be called from the
    /// browser process.
    pub fn view_source(&self) {
        if let Some(view_source) = self.as_ref().view_source {
            unsafe {
                view_source(self.0);
            }
        }
    }
    /// Retrieve this frame's HTML source as a string sent to the specified
    /// visitor.
    pub fn get_source(&self, visitor: Box<dyn StringVisitor>) {
        if let Some(get_source) = self.as_ref().get_source {
            let visitor = StringVisitorWrapper::wrap(visitor);
            unsafe {
                get_source(self.0, visitor);
            }
        }
    }
    /// Retrieve this frame's display text as a string sent to the specified
    /// visitor.
    pub fn get_text(&self, visitor: Box<dyn StringVisitor>) {
        if let Some(get_text) = self.as_ref().get_text {
            let visitor = StringVisitorWrapper::wrap(visitor);
            unsafe {
                get_text(self.0, visitor);
            }
        }
    }
    /// Load the request represented by the |request| object.
    pub fn load_request(&mut self, request: &Request) {
        if let Some(load_request) = self.as_ref().load_request {
            unsafe {
                load_request(self.0, request.as_ptr());
            }
        }
    }
    /// Load the specified `url`.
    pub fn load_url(&mut self, url: &str) {
        if let Some(load_url) = self.as_ref().load_url {
            unsafe {
                load_url(self.0, CefString::new(url).as_ref());
            }
        }
    }
    /// Load the contents of `string_val` with the specified dummy `url`. `url`
    /// should have a standard scheme (for example, http scheme) or behaviors like
    /// link clicks and web security restrictions may not behave as expected.
    pub fn load_string(&mut self, string_val: &str, url: &str) {
        if let Some(load_string) = self.as_ref().load_string {
            unsafe {
                load_string(
                    self.0,
                    CefString::new(string_val).as_ref(),
                    CefString::new(url).as_ref(),
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
        if let Some(execute_java_script) = self.as_ref().execute_java_script {
            unsafe {
                execute_java_script(
                    self.0,
                    CefString::new(code).as_ref(),
                    CefString::new(script_url).as_ref(),
                    start_line,
                );
            }
        }
    }
    /// Returns true if this is the main (top-level) frame.
    pub fn is_main(&self) -> bool {
        if let Some(is_main) = self.as_ref().is_main {
            unsafe { is_main(self.0) != 0 }
        } else {
            false
        }
    }
    /// Returns true if this is the focused frame.
    pub fn is_focused(&self) -> bool {
        if let Some(is_focused) = self.as_ref().is_focused {
            unsafe { is_focused(self.0) != 0 }
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
        if let Some(get_name) = self.as_ref().get_name {
            let name = unsafe { get_name(self.0) };
            let result = CefString::copy_raw_to_string(name);
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
        if let Some(get_identifier) = self.as_ref().get_identifier {
            let id = unsafe { get_identifier(self.0) };
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
        if let Some(get_parent) = self.as_ref().get_parent {
            let parent = unsafe { get_parent(self.0) };
            if parent.is_null() {
                None
            } else {
                Some(Frame::from(parent))
            }
        } else {
            None
        }
    }
    /// Returns the URL currently loaded in this frame.
    pub fn get_url(&self) -> String {
        if let Some(get_url) = self.as_ref().get_url {
            let url = unsafe { get_url(self.0) };
            let result = CefString::copy_raw_to_string(url);
            if let Some(result) = result {
                unsafe {
                    cef_string_userfree_utf16_free(url);
                }
                result
            } else {
                "".to_owned()
            }
        } else {
            "".to_owned()
        }
    }
    /// Returns the browser that this frame belongs to.
    pub fn get_browser(&self) -> Browser {
        let browser = unsafe { self.as_ref().get_browser.unwrap()(self.0) };
        if browser.is_null() {
            panic!("CEF: Frame without a browser!")
        }
        Browser::from(browser)
    }
    /// Get the V8 context associated with the frame. This function can only be
    /// called from the render process.
    pub fn get_v8context(&self) -> V8Context {
        let context = unsafe { self.as_ref().get_v8context.unwrap()(self.0) };
        if context.is_null() {
            panic!("CEF: Frame without a V8 context!")
        }
        V8Context::from(context)
    }
    /// Visit the DOM document. This function can only be called from the render
    /// process.
    pub fn visit_dom(&self, visitor: Box<dyn DOMVisitor>) {
        if let Some(visit_dom) = self.as_ref().visit_dom {
            let visitor = DOMVisitorWrapper::wrap(visitor);
            unsafe { visit_dom(self.0, visitor) };
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
        let urlrequest = unsafe {
            (&*self.0).create_urlrequest.unwrap()(
                self.0,
                request.as_ptr(),
                URLRequestClientWrapper::wrap(client),
            )
        };
        URLRequest::from(urlrequest)
    }
}
