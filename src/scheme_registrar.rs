use cef_sys::{cef_scheme_options_t, cef_scheme_registrar_t};
use crate::string::CefString;
use bitflags::bitflags;

bitflags!{
    /// Configuration options for registering a custom scheme.
    /// These values are used when calling [SchemeRegistrar::add_custom_scheme].
    pub struct SchemeOptions: crate::CEnumType {
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
        const STANDARD = cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD as crate::CEnumType;
        /// If `Local` is set the scheme will be treated with the same
        /// security rules as those applied to "file" URLs. Normal pages cannot link to
        /// or access local URLs. Also, by default, local URLs can only perform
        /// XMLHttpRequest calls to the same URL (origin + path) that originated the
        /// request. To allow XMLHttpRequest calls from a local URL to other URLs with
        /// the same origin set the CefSettings.file_access_from_file_urls_allowed
        /// value to true. To allow XMLHttpRequest calls from a local URL to all
        /// origins set the CefSettings.universal_access_from_file_urls_allowed value
        /// to true.
        const LOCAL = cef_scheme_options_t::CEF_SCHEME_OPTION_LOCAL as crate::CEnumType;
        /// If `DisplayIsolated` is set the scheme can only be
        /// displayed from other content hosted with the same scheme. For example,
        /// pages in other origins cannot create iframes or hyperlinks to URLs with the
        /// scheme. For schemes that must be accessible from other schemes don't set
        /// this, set `CORSEnabled`, and use CORS
        /// "Access-Control-Allow-Origin" headers to further restrict access.
        const DISPLAY_ISOLATED = cef_scheme_options_t::CEF_SCHEME_OPTION_DISPLAY_ISOLATED as crate::CEnumType;
        /// If `Secure` is set the scheme will be treated with the same
        /// security rules as those applied to "https" URLs. For example, loading this
        /// scheme from other secure schemes will not trigger mixed content warnings.
        const SECURE = cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE as crate::CEnumType;
        /// If `CORSEnabled` is set the scheme can be sent CORS
        /// requests. This value should be set in most cases where
        /// `Standard` is set.
        const CORS_ENABLED = cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED as crate::CEnumType;
        /// If `CSPBypassing` is set the scheme can bypass Content-
        /// Security-Policy (CSP) checks. This value should not be set in most cases
        /// where `Standard` is set.
        const CSP_BYPASSING = cef_scheme_options_t::CEF_SCHEME_OPTION_CSP_BYPASSING as crate::CEnumType;
        /// If `FetchEnabled` is set the scheme can perform Fetch API
        /// requests.
        const FETCH_ENABLED = cef_scheme_options_t::CEF_SCHEME_OPTION_FETCH_ENABLED as crate::CEnumType;
    }
}

/// Structure that manages custom scheme registrations.
pub struct SchemeRegistrar(*mut cef_scheme_registrar_t);
owned_casts!(impl for SchemeRegistrar = *mut cef_scheme_registrar_t);

impl SchemeRegistrar {
    pub unsafe fn from_ptr_unchecked(ptr: *mut cef_scheme_registrar_t) -> SchemeRegistrar {
        SchemeRegistrar(ptr)
    }

    pub(crate) unsafe fn from_ptr_ptr<'a>(ptr: *mut *mut cef_scheme_registrar_t) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    /// Register a custom scheme. This function should not be called for the built-
    /// in HTTP, HTTPS, FILE, FTP, ABOUT and DATA schemes.
    ///
    /// This function may be called on any thread. It should only be called once
    /// per unique `scheme_name` value. If `scheme_name` is already registered or
    /// if an error occurs this function will return false.
    pub fn add_custom_scheme(&self, scheme_name: &str, options: SchemeOptions) -> bool {
        unsafe {
            ((*self.0).add_custom_scheme.unwrap())(
                self.0,
                CefString::new(scheme_name).as_ptr(),
                options.bits() as _,
            ) != 0
        }
    }
}
