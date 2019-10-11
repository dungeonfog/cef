use cef_sys::{
    cef_base_ref_counted_t, cef_post_data_create, cef_post_data_element_create,
    cef_post_data_element_t, cef_post_data_t, cef_postdataelement_type_t, cef_referrer_policy_t,
    cef_request_create, cef_request_t, cef_resource_type_t, cef_string_userfree_utf16_free,
};
use num_enum::UnsafeFromPrimitive;
use std::{collections::HashMap, convert::TryFrom, ptr::null_mut};

use crate::{load_handler::TransitionType, multimap::MultiMap, string::CefString};

/// Policy for how the Referrer HTTP header value will be sent during navigation.
/// if the `--no-referrers` command-line flag is specified then the policy value
/// will be ignored and the Referrer value will never be sent.
/// Must be kept synchronized with `net::URLRequest::ReferrerPolicy` from Chromium.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum ReferrerPolicy {
    /// Clear the referrer header if the header value is HTTPS but the request
    /// destination is HTTP. This is the default behavior.1
    Default = cef_referrer_policy_t::REFERRER_POLICY_DEFAULT as i32,
    /// A slight variant on CLEAR_REFERRER_ON_TRANSITION_FROM_SECURE_TO_INSECURE (Default):
    /// if the request destination is HTTP, an HTTPS referrer will be cleared. if
    /// the request's destination is cross-origin with the referrer (but does not
    /// downgrade), the referrer's granularity will be stripped down to an origin
    /// rather than a full URL. Same-origin requests will send the full referrer.
    ReduceReferrerGranularityOnTransitionCrossOrigin = cef_referrer_policy_t::REFERRER_POLICY_REDUCE_REFERRER_GRANULARITY_ON_TRANSITION_CROSS_ORIGIN as i32,
    /// Strip the referrer down to an origin when the origin of the referrer is
    /// different from the destination's origin.
    OriginOnlyOnTransitionCrossOrigin = cef_referrer_policy_t::REFERRER_POLICY_ORIGIN_ONLY_ON_TRANSITION_CROSS_ORIGIN as i32,
    /// Never change the referrer.
    NeverClearReferrer = cef_referrer_policy_t::REFERRER_POLICY_NEVER_CLEAR_REFERRER as i32,
    /// Strip the referrer down to the origin regardless of the redirect location.
    Origin = cef_referrer_policy_t::REFERRER_POLICY_ORIGIN as i32,
    /// Clear the referrer when the request's referrer is cross-origin with the
    /// request's destination.
    ClearReferrerOnTransitionCrossOrigin = cef_referrer_policy_t::REFERRER_POLICY_CLEAR_REFERRER_ON_TRANSITION_CROSS_ORIGIN as i32,
    /// Strip the referrer down to the origin, but clear it entirely if the
    /// referrer value is HTTPS and the destination is HTTP.
    OriginClearOnTransitionFromSecureToInsecure = cef_referrer_policy_t::REFERRER_POLICY_ORIGIN_CLEAR_ON_TRANSITION_FROM_SECURE_TO_INSECURE as i32,
    /// Always clear the referrer regardless of the request destination.
    NoReferrer = cef_referrer_policy_t::REFERRER_POLICY_NO_REFERRER as i32,
}

impl Into<cef_referrer_policy_t::Type> for ReferrerPolicy {
    fn into(self) -> cef_referrer_policy_t::Type {
        unsafe { std::mem::transmute(self as i32) }
    }
}

/// Flags used to customize the behavior of [URLRequest].
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum URLRequestFlags {
    /// if set the cache will be skipped when handling the request. Setting this
    /// value is equivalent to specifying the "Cache-Control: no-cache" request
    /// header. Setting this value in combination with OnlyFromCache will
    /// cause the request to fail.
    SkipCache = 0,
    /// if set the request will fail if it cannot be served from the cache (or some
    /// equivalent local store). Setting this value is equivalent to specifying the
    /// "Cache-Control: only-if-cached" request header. Setting this value in
    /// combination with SkipCache or DisableCache will cause the
    /// request to fail.
    OnlyFromCache = 1,
    /// if set the cache will not be used at all. Setting this value is equivalent
    /// to specifying the "Cache-Control: no-store" request header. Setting this
    /// value in combination with OnlyFromCache will cause the request to
    /// fail.
    DisableCache = 2,
    /// if set user name, password, and cookies may be sent with the request, and
    /// cookies may be saved from the response.
    AllowStoredCredentials = 3,
    /// if set upload progress events will be generated when a request has a body.
    ReportUploadProgress = 4,
    /// if set the [URLRequestClient::on_download_data] method will not be called.
    NoDownloadData = 5,
    /// if set 5XX redirect errors will be propagated to the observer instead of
    /// automatically re-tried. This currently only applies for requests
    /// originated in the browser process.
    NoRetryOn5xx = 6,
    /// if set 3XX responses will cause the fetch to halt immediately rather than
    /// continue through the redirect.
    StopOnRedirect = 7,
}

impl URLRequestFlags {
    pub(crate) fn to_bitfield(flags: &Vec<URLRequestFlags>) -> i32 {
        flags
            .iter()
            .fold(0, |flags, flag| flags | (1 << (*flag) as i32))
    }
    pub(crate) fn from_bitfield(bitfield: i32) -> Vec<URLRequestFlags> {
        [
            URLRequestFlags::SkipCache,
            URLRequestFlags::OnlyFromCache,
            URLRequestFlags::DisableCache,
            URLRequestFlags::AllowStoredCredentials,
            URLRequestFlags::ReportUploadProgress,
            URLRequestFlags::NoDownloadData,
            URLRequestFlags::NoRetryOn5xx,
            URLRequestFlags::StopOnRedirect,
        ]
        .into_iter()
        .filter(|flag| bitfield & (1 << (**flag) as i32) != 0)
        .cloned()
        .collect()
    }
}

/// Resource type for a request.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum ResourceType {
    /// Top level page.
    MainFrame = cef_resource_type_t::RT_MAIN_FRAME as i32,
    /// Frame or iframe.
    SubFrame = cef_resource_type_t::RT_SUB_FRAME as i32,
    /// CSS stylesheet.
    Stylesheet = cef_resource_type_t::RT_STYLESHEET as i32,
    /// External script.
    Script = cef_resource_type_t::RT_SCRIPT as i32,
    /// Image (jpg/gif/png/etc).
    Image = cef_resource_type_t::RT_IMAGE as i32,
    /// Font.
    FontResource = cef_resource_type_t::RT_FONT_RESOURCE as i32,
    /// Some other subresource. This is the default type if the actual type is
    /// unknown.
    SubResource = cef_resource_type_t::RT_SUB_RESOURCE as i32,
    /// Object (or embed) tag for a plugin, or a resource that a plugin requested.
    Object = cef_resource_type_t::RT_OBJECT as i32,
    /// Media resource.
    Media = cef_resource_type_t::RT_MEDIA as i32,
    /// Main resource of a dedicated worker.
    Worker = cef_resource_type_t::RT_WORKER as i32,
    /// Main resource of a shared worker.
    SharedWorker = cef_resource_type_t::RT_SHARED_WORKER as i32,
    /// Explicitly requested prefetch.
    Prefetch = cef_resource_type_t::RT_PREFETCH as i32,
    /// Favicon.
    Favicon = cef_resource_type_t::RT_FAVICON as i32,
    /// XMLHttpRequest.
    XHR = cef_resource_type_t::RT_XHR as i32,
    /// A request for a <ping>
    Ping = cef_resource_type_t::RT_PING as i32,
    /// Main resource of a service worker.
    ServiceWorker = cef_resource_type_t::RT_SERVICE_WORKER as i32,
    /// A report of Content Security Policy violations.
    CSPReport = cef_resource_type_t::RT_CSP_REPORT as i32,
    /// A resource that a plugin requested.
    PluginResource = cef_resource_type_t::RT_PLUGIN_RESOURCE as i32,
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum PostDataElementType {
    Empty = cef_postdataelement_type_t::PDE_TYPE_EMPTY as i32,
    Bytes = cef_postdataelement_type_t::PDE_TYPE_BYTES as i32,
    File = cef_postdataelement_type_t::PDE_TYPE_FILE as i32,
}

ref_counted_ptr! {
    /// Structure used to represent a web request. The functions of this structure
    /// may be called on any thread.
    #[derive(Clone)]
    pub struct Request(*mut cef_request_t);
}

unsafe impl Send for Request {}
unsafe impl Sync for Request {}

impl Request {
    /// Create a new Request object.
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_request_create()) }
    }

    /// Returns true if this object is read-only.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .and_then(|is_read_only| Some(unsafe { is_read_only(self.0.as_ptr()) != 0 }))
            .unwrap_or(true)
    }
    /// Get the fully qualified URL.
    pub fn get_url(&self) -> String {
        self.0
            .get_url
            .and_then(|get_url| {
                let mut s = unsafe { get_url(self.0.as_ptr()) };
                let result = CefString::copy_raw_to_string(s);
                unsafe {
                    cef_string_userfree_utf16_free(s);
                }
                result
            })
            .unwrap_or("".to_owned())
    }
    /// Set the fully qualified URL.
    pub fn set_url(&mut self, url: &str) {
        if let Some(set_url) = self.0.set_url {
            unsafe {
                set_url(self.0.as_ptr(), CefString::new(url).as_ref());
            }
        }
    }
    /// Get the request function type. The value will default to POST if post data
    /// is provided and GET otherwise.
    pub fn get_method(&self) -> String {
        self.0
            .get_method
            .and_then(|get_method| {
                let s = unsafe { get_method(self.0.as_ptr()) };
                let result = CefString::copy_raw_to_string(s);
                unsafe {
                    cef_string_userfree_utf16_free(s);
                }
                result
            })
            .unwrap_or("GET".to_owned())
    }
    /// Set the request function type.
    pub fn set_method(&mut self, method: &str) {
        if let Some(set_method) = self.0.set_method {
            unsafe {
                set_method(self.0.as_ptr(), CefString::new(method).as_ref());
            }
        }
    }
    /// Set the referrer URL and policy. if `Some` the referrer URL must be fully
    /// qualified with an HTTP or HTTPS scheme component. Any username, password or
    /// ref component will be removed.
    pub fn set_referrer(&mut self, referrer_url: Option<&str>, policy: ReferrerPolicy) {
        if let Some(set_referrer) = self.0.set_referrer {
            if let Some(referrer_url) = referrer_url {
                unsafe {
                    set_referrer(
                        self.0.as_ptr(),
                        CefString::new(referrer_url).as_ref(),
                        policy.into(),
                    );
                }
            }
        }
    }
    /// Get the referrer URL.
    pub fn get_referrer_url(&self) -> String {
        self.0
            .get_referrer_url
            .and_then(|get_referrer_url| {
                let s = unsafe { get_referrer_url(self.0.as_ptr()) };
                let result = CefString::copy_raw_to_string(s);
                unsafe {
                    cef_string_userfree_utf16_free(s);
                }
                result
            })
            .unwrap_or("".to_owned())
    }
    /// Get the referrer policy.
    pub fn get_referrer_policy(&self) -> ReferrerPolicy {
        self.0
            .get_referrer_policy
            .and_then(|get_referrer_policy| {
                Some(unsafe {
                    ReferrerPolicy::from_unchecked(get_referrer_policy(self.0.as_ptr()) as i32)
                })
            })
            .unwrap_or(ReferrerPolicy::Default)
    }
    /// Get the post data.
    pub fn get_post_data(&self) -> PostData {
        let get_post_data = self.0.get_post_data.unwrap();
        unsafe { PostData::from_ptr_unchecked(get_post_data(self.0.as_ptr())) }
    }
    /// Set the post data.
    pub fn set_post_data(&mut self, post_data: PostData) {
        if let Some(set_post_data) = self.0.set_post_data {
            unsafe {
                set_post_data(self.0.as_ptr(), post_data.into_raw());
            }
        }
    }
    /// Get the header values. Will not include the Referer value if any.
    pub fn get_header_map(&self) -> HashMap<String, Vec<String>> {
        if let Some(get_header_map) = self.0.get_header_map {
            let mut map = MultiMap::new();
            unsafe { get_header_map(self.0.as_ptr(), map.as_ptr()) };
            map.into()
        } else {
            HashMap::new()
        }
    }
    /// Returns the first header value for `name` or None if not found.
    /// Will not return the Referer value if any. Use [Request::get_header_map] instead if
    /// `name` might have multiple values.
    pub fn get_header_by_name(&self, name: &str) -> Option<String> {
        if let Some(get_header_by_name) = self.0.get_header_by_name {
            let header =
                unsafe { get_header_by_name(self.0.as_ptr(), CefString::new(name).as_ref()) };
            let result = CefString::copy_raw_to_string(header);
            if result.is_some() {
                unsafe {
                    cef_string_userfree_utf16_free(header);
                }
            }
            result
        } else {
            None
        }
    }
    /// Set the header `name` to `value`. if `overwrite` is true any existing
    /// values will be replaced with the new value. if `overwrite` is false any
    /// existing values will not be overwritten. The Referer value cannot be set
    /// using this function.
    pub fn set_header_by_name(&mut self, name: &str, value: &str, overwrite: bool) {
        if let Some(set_header_by_name) = self.0.set_header_by_name {
            unsafe {
                set_header_by_name(
                    self.0.as_ptr(),
                    CefString::new(name).as_ref(),
                    CefString::new(value).as_ref(),
                    overwrite as i32,
                );
            }
        }
    }
    /// Set all values at one time.
    pub fn set(
        &mut self,
        url: &str,
        method: &str,
        post_data: PostData,
        header_map: HashMap<String, Vec<String>>,
    ) {
        if let Some(set) = self.0.set {
            let url = CefString::new(url);
            let method = CefString::new(method);
            let header_map = MultiMap::from(&header_map);

            unsafe {
                set(
                    self.0.as_ptr(),
                    url.as_ref(),
                    method.as_ref(),
                    post_data.into_raw(),
                    header_map.as_ptr(),
                );
            }
        }
    }
    /// Get the flags used in combination with [URLRequest]. See
    /// [URLRequestFlags] for supported values.
    pub fn get_flags(&self) -> Vec<URLRequestFlags> {
        if let Some(get_flags) = self.0.get_flags {
            URLRequestFlags::from_bitfield(unsafe { get_flags(self.0.as_ptr()) })
        } else {
            Vec::new()
        }
    }
    /// Set the flags used in combination with [URLRequest]. See
    /// [URLRequestFlags] for supported values.
    pub fn set_flags(&mut self, flags: &Vec<URLRequestFlags>) {
        if let Some(set_flags) = self.0.set_flags {
            unsafe {
                set_flags(self.0.as_ptr(), URLRequestFlags::to_bitfield(flags));
            }
        }
    }
    /// Get the URL to the first party for cookies used in combination with
    /// [URLRequest].
    pub fn get_first_party_for_cookies(&self) -> String {
        if let Some(get_first_party_for_cookies) = self.0.get_first_party_for_cookies {
            let url = unsafe { get_first_party_for_cookies(self.0.as_ptr()) };
            let result = CefString::copy_raw_to_string(url);
            if result.is_some() {
                unsafe {
                    cef_string_userfree_utf16_free(url);
                }
            }
            result.unwrap_or_else(|| "".to_owned())
        } else {
            "".to_owned()
        }
    }
    /// Set the URL to the first party for cookies used in combination with
    /// [URLRequest].
    pub fn set_first_party_for_cookies(&mut self, url: &str) {
        if let Some(set_first_party_for_cookies) = self.0.set_first_party_for_cookies {
            unsafe {
                set_first_party_for_cookies(self.0.as_ptr(), CefString::new(url).as_ref());
            }
        }
    }
    /// Get the resource type for this request. Only available in the browser
    /// process.
    pub fn get_resource_type(&self) -> ResourceType {
        unsafe {
            ResourceType::from_unchecked(((&*self.0.as_ptr()).get_resource_type).unwrap()(
                self.0.as_ptr(),
            ) as i32)
        }
    }
    /// Get the transition type for this request. Only available in the browser
    /// process and only applies to requests that represent a main frame or sub-
    /// frame navigation.
    pub fn get_transition_type(&self) -> TransitionType {
        TransitionType::try_from(unsafe {
            (&*self.0.as_ptr()).get_transition_type.unwrap()(self.0.as_ptr()).0
        })
        .unwrap()
    }
    /// Returns the globally unique identifier for this request or 0 if not
    /// specified. Can be used by [ResourceRequestHandler] implementations in
    /// the browser process to track a single request across multiple callbacks.
    pub fn get_identifier(&self) -> u64 {
        if let Some(get_identifier) = self.0.get_identifier {
            unsafe { get_identifier(self.0.as_ptr()) }
        } else {
            0
        }
    }
}

ref_counted_ptr! {
    /// Structure used to represent post data for a web request. The functions of
    /// this structure may be called on any thread.
    #[derive(Clone)]
    pub struct PostData(*mut cef_post_data_t);
}

unsafe impl Send for PostData {}
unsafe impl Sync for PostData {}

impl PostData {
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_post_data_create()) }
    }

    /// Returns true if this object is read-only.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .and_then(|is_read_only| Some(unsafe { is_read_only(self.as_ptr()) != 0 }))
            .unwrap_or(true)
    }
    /// Returns true if the underlying POST data includes elements that are not
    /// represented by this [PostData] object (for example, multi-part file
    /// upload data). Modifying [PostData] objects with excluded elements may
    /// result in the request failing.
    pub fn has_excluded_elements(&self) -> bool {
        self.0
            .has_excluded_elements
            .and_then(|has_excluded_elements| {
                Some(unsafe { has_excluded_elements(self.as_ptr()) != 0 })
            })
            .unwrap_or(false)
    }
    /// Returns the number of existing post data elements.
    pub fn get_element_count(&self) -> usize {
        self.0
            .get_element_count
            .and_then(|get_element_count| Some(unsafe { get_element_count(self.as_ptr()) }))
            .unwrap_or(0)
    }
    /// Retrieve the post data elements.
    pub fn get_elements(&self) -> Vec<PostDataElement> {
        let mut count = self.get_element_count();
        if count > 0 {
            if let Some(get_elements) = self.0.get_elements {
                let mut elements = vec![null_mut(); count];
                unsafe {
                    get_elements(self.as_ptr(), &mut count, elements.as_mut_ptr());
                }
                elements
                    .into_iter()
                    .map(|p| unsafe { PostDataElement::from_ptr_unchecked(p) })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    /// Remove the specified post data element. Returns true if the removal
    /// succeeds.
    pub fn remove_element(&mut self, element: &PostDataElement) -> bool {
        if let Some(remove_element) = self.0.remove_element {
            unsafe { remove_element(self.as_ptr(), element.as_ptr()) != 0 }
        } else {
            false
        }
    }
    /// Add the specified post data element. Returns true if the add succeeds.
    pub fn add_element(&mut self, element: &PostDataElement) -> bool {
        if let Some(add_element) = self.0.add_element {
            unsafe { add_element(self.as_ptr(), element.as_ptr()) != 0 }
        } else {
            false
        }
    }
    /// Remove all existing post data elements.
    pub fn remove_elements(&mut self) {
        if let Some(remove_elements) = self.0.remove_elements {
            unsafe {
                remove_elements(self.as_ptr());
            }
        }
    }
}

ref_counted_ptr! {
    /// Structure used to represent a single element in the request post data. The
    /// functions of this structure may be called on any thread.
    #[derive(Clone)]
    pub struct PostDataElement(*mut cef_post_data_element_t);
}

unsafe impl Send for PostDataElement {}
unsafe impl Sync for PostDataElement {}

impl PostDataElement {
    /// Create a new [PostDataElement] object.
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_post_data_element_create()) }
    }

    /// Returns true if this object is read-only.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .and_then(|is_read_only| Some(unsafe { is_read_only(self.as_ptr()) != 0 }))
            .unwrap_or(true)
    }
    /// Remove all contents from the post data element.
    pub fn set_to_empty(&mut self) {
        if let Some(set_to_empty) = self.0.set_to_empty {
            unsafe {
                set_to_empty(self.as_ptr());
            }
        }
    }
    /// The post data element will represent a file.
    pub fn set_to_file(&mut self, file_name: &str) {
        if let Some(set_to_file) = self.0.set_to_file {
            unsafe {
                set_to_file(self.0.as_ptr(), CefString::new(file_name).as_ref());
            }
        }
    }
    /// The post data element will represent bytes.  The bytes passed in will be
    /// copied.
    pub fn set_to_bytes(&mut self, bytes: &[u8]) {
        if let Some(set_to_bytes) = self.0.set_to_bytes {
            unsafe {
                set_to_bytes(
                    self.0.as_ptr(),
                    bytes.len(),
                    bytes.as_ptr() as *const std::ffi::c_void,
                );
            }
        }
    }
    /// Return the type of this post data element.
    pub fn get_type(&self) -> PostDataElementType {
        if let Some(get_type) = self.0.get_type {
            unsafe { PostDataElementType::from_unchecked(get_type(self.as_ptr()) as i32) }
        } else {
            PostDataElementType::Empty
        }
    }
    /// Return the file name.
    pub fn get_file(&self) -> String {
        let name = unsafe { (&*self.0).get_file.unwrap()(self.as_ptr()) };
        if let Some(result) = CefString::copy_raw_to_string(name) {
            unsafe {
                cef_string_userfree_utf16_free(name);
            }
            result
        } else {
            "".to_owned()
        }
    }
    /// Return the number of bytes.
    pub fn get_bytes_count(&self) -> usize {
        if let Some(get_bytes_count) = self.0.get_bytes_count {
            unsafe { get_bytes_count(self.as_ptr()) }
        } else {
            0
        }
    }
    /// Return the bytes.
    pub fn get_bytes(&self) -> Vec<u8> {
        let size = self.get_bytes_count();
        if size > 0 {
            if let Some(get_bytes) = self.0.get_bytes {
                let mut buffer = vec![0; size];
                unsafe {
                    get_bytes(
                        self.as_ptr(),
                        size,
                        buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    );
                }
                buffer.to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
}
