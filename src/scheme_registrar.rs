use cef_sys::{cef_scheme_options_t, cef_scheme_registrar_t};

use crate::string::CefString;

/// Configuration options for registering a custom scheme.
/// These values are used when calling [SchemeRegistrar::add_custom_scheme].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SchemeOptions {
    /// If `Standard` is set the scheme will be treated as a
    /// standard scheme. Standard schemes are subject to URL canonicalization and
    /// parsing rules as defined in the [Common Internet Scheme Syntax RFC 1738](http://www.ietf.org/rfc/rfc1738.txt)
    /// Section 3.1
    ///
    /// In particular, the syntax for standard scheme URLs must be of the form:
    /// <pre>
    ///  [scheme]://[username]:[password]@[host]:[port]/[url-path]
    /// </pre> Standard scheme URLs must have a host component that is a fully
    /// qualified domain name as defined in Section 3.5 of RFC 1034 \[13\] and
    /// Section 2.1 of RFC 1123. These URLs will be canonicalized to
    /// "scheme://host/path" in the simplest case and
    /// "scheme://username:password@host:port/path" in the most explicit case. For
    /// example, "scheme:host/path" and "scheme:///host/path" will both be
    /// canonicalized to "scheme://host/path". The origin of a standard scheme URL
    /// is the combination of scheme, host and port (i.e., "scheme://host:port" in
    /// the most explicit case).
    ///
    /// For non-standard scheme URLs only the "scheme:" component is parsed and
    /// canonicalized. The remainder of the URL will be passed to the handler as-
    /// is. For example, "scheme:///some%20text" will remain the same. Non-standard
    /// scheme URLs cannot be used as a target for form submission.
    Standard,
    /// If `Local` is set the scheme will be treated with the same
    /// security rules as those applied to "file" URLs. Normal pages cannot link to
    /// or access local URLs. Also, by default, local URLs can only perform
    /// XMLHttpRequest calls to the same URL (origin + path) that originated the
    /// request. To allow XMLHttpRequest calls from a local URL to other URLs with
    /// the same origin set the CefSettings.file_access_from_file_urls_allowed
    /// value to true. To allow XMLHttpRequest calls from a local URL to all
    /// origins set the CefSettings.universal_access_from_file_urls_allowed value
    /// to true.
    Local,
    /// If `DisplayIsolated` is set the scheme can only be
    /// displayed from other content hosted with the same scheme. For example,
    /// pages in other origins cannot create iframes or hyperlinks to URLs with the
    /// scheme. For schemes that must be accessible from other schemes don't set
    /// this, set `CORSEnabled`, and use CORS
    /// "Access-Control-Allow-Origin" headers to further restrict access.
    DisplayIsolated,
    /// If `Secure` is set the scheme will be treated with the same
    /// security rules as those applied to "https" URLs. For example, loading this
    /// scheme from other secure schemes will not trigger mixed content warnings.
    Secure,
    /// If `CORSEnabled` is set the scheme can be sent CORS
    /// requests. This value should be set in most cases where
    /// `Standard` is set.
    CORSEnabled,
    /// If `CSPBypassing` is set the scheme can bypass Content-
    /// Security-Policy (CSP) checks. This value should not be set in most cases
    /// where `Standard` is set.
    CSPBypassing,
    /// If `FetchEnabled` is set the scheme can perform Fetch API
    /// requests.
    FetchEnabled,
}

#[doc(hidden)]
impl Into<cef_scheme_options_t::Type> for SchemeOptions {
    fn into(self) -> cef_scheme_options_t::Type {
        match self {
            SchemeOptions::Standard => cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD,
            SchemeOptions::Local => cef_scheme_options_t::CEF_SCHEME_OPTION_LOCAL,
            SchemeOptions::DisplayIsolated => {
                cef_scheme_options_t::CEF_SCHEME_OPTION_DISPLAY_ISOLATED
            }
            SchemeOptions::Secure => cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE,
            SchemeOptions::CORSEnabled => cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED,
            SchemeOptions::CSPBypassing => cef_scheme_options_t::CEF_SCHEME_OPTION_CSP_BYPASSING,
            SchemeOptions::FetchEnabled => cef_scheme_options_t::CEF_SCHEME_OPTION_FETCH_ENABLED,
        }
    }
}

/// Structure that manages custom scheme registrations.
pub struct SchemeRegistrar(*mut cef_scheme_registrar_t);

impl SchemeRegistrar {
    pub unsafe fn from_ptr_unchecked(ptr: *mut cef_scheme_registrar_t) -> SchemeRegistrar {
        SchemeRegistrar(ptr)
    }

    /// Register a custom scheme. This function should not be called for the built-
    /// in HTTP, HTTPS, FILE, FTP, ABOUT and DATA schemes.
    ///
    /// This function may be called on any thread. It should only be called once
    /// per unique `scheme_name` value. If `scheme_name` is already registered or
    /// if an error occurs this function will return false.
    pub fn add_custom_scheme(&self, scheme_name: &str, options: &[SchemeOptions]) -> bool {
        let options: std::os::raw::c_int = options.iter().fold(0, |flags, option| {
            flags | (<SchemeOptions as Into<cef_scheme_options_t::Type>>::into(*option))
        });
        unsafe {
            ((*self.0).add_custom_scheme.unwrap())(
                self.0,
                CefString::new(scheme_name).as_ptr(),
                options,
            ) != 0
        }
    }
}

unsafe impl Send for SchemeRegistrar {}
unsafe impl Sync for SchemeRegistrar {}

impl Drop for SchemeRegistrar {
    fn drop(&mut self) {
        unsafe { ((*self.0).base.del.unwrap())(&mut (*self.0).base) }
    }
}
