use cef_sys::cef_sslstatus_t;
use cef_sys::{_cef_sslinfo_t, cef_cert_status_t, cef_is_cert_status_error, cef_ssl_version_t, cef_ssl_content_status_t};
use bitflags::bitflags;
use crate::x509_certificate::X509Certificate;

bitflags!{
    /// Supported certificate status code values. See net\cert\cert_status_flags.h
    /// for more information. CERT_STATUS_NONE is new in CEF because we use an
    /// enum while cert_status_flags.h uses a typedef and static const variables.
    #[derive(Default)]
    pub struct CertStatus: crate::CEnumType {
        const NONE = cef_cert_status_t::CERT_STATUS_NONE.0;
        const COMMON_NAME_INVALID = cef_cert_status_t::CERT_STATUS_COMMON_NAME_INVALID.0;
        const DATE_INVALID = cef_cert_status_t::CERT_STATUS_DATE_INVALID.0;
        const AUTHORITY_INVALID = cef_cert_status_t::CERT_STATUS_AUTHORITY_INVALID.0;
        const NO_REVOCATION_MECHANISM = cef_cert_status_t::CERT_STATUS_NO_REVOCATION_MECHANISM.0;
        const UNABLE_TO_CHECK_REVOCATION = cef_cert_status_t::CERT_STATUS_UNABLE_TO_CHECK_REVOCATION.0;
        const REVOKED = cef_cert_status_t::CERT_STATUS_REVOKED.0;
        const INVALID = cef_cert_status_t::CERT_STATUS_INVALID.0;
        const WEAK_SIGNATURE_ALGORITHM = cef_cert_status_t::CERT_STATUS_WEAK_SIGNATURE_ALGORITHM.0;
        const NON_UNIQUE_NAME = cef_cert_status_t::CERT_STATUS_NON_UNIQUE_NAME.0;
        const WEAK_KEY = cef_cert_status_t::CERT_STATUS_WEAK_KEY.0;
        const PINNED_KEY_MISSING = cef_cert_status_t::CERT_STATUS_PINNED_KEY_MISSING.0;
        const NAME_CONSTRAINT_VIOLATION = cef_cert_status_t::CERT_STATUS_NAME_CONSTRAINT_VIOLATION.0;
        const VALIDITY_TOO_LONG = cef_cert_status_t::CERT_STATUS_VALIDITY_TOO_LONG.0;
        const IS_EV = cef_cert_status_t::CERT_STATUS_IS_EV.0;
        const REV_CHECKING_ENABLED = cef_cert_status_t::CERT_STATUS_REV_CHECKING_ENABLED.0;
        const SHA1_SIGNATURE_PRESENT = cef_cert_status_t::CERT_STATUS_SHA1_SIGNATURE_PRESENT.0;
        const CT_COMPLIANCE_FAILED = cef_cert_status_t::CERT_STATUS_CT_COMPLIANCE_FAILED.0;
    }
}

// Supported SSL version values. See net/ssl/ssl_connection_status_flags.h
// for more information.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SSLVersion {
    Unknown = cef_ssl_version_t::SSL_CONNECTION_VERSION_UNKNOWN as isize,
    SSL2 = cef_ssl_version_t::SSL_CONNECTION_VERSION_SSL2 as isize,
    SSL3 = cef_ssl_version_t::SSL_CONNECTION_VERSION_SSL3 as isize,
    TLS1 = cef_ssl_version_t::SSL_CONNECTION_VERSION_TLS1 as isize,
    TLS1_1 = cef_ssl_version_t::SSL_CONNECTION_VERSION_TLS1_1 as isize,
    TLS1_2 = cef_ssl_version_t::SSL_CONNECTION_VERSION_TLS1_2 as isize,
    TLS1_3 = cef_ssl_version_t::SSL_CONNECTION_VERSION_TLS1_3 as isize,
    QUIC = cef_ssl_version_t::SSL_CONNECTION_VERSION_QUIC as isize,
}

impl SSLVersion {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

bitflags!{
    // Supported SSL content status flags. See content/public/common/ssl_status.h
    // for more information.
    pub struct ContentStatus: crate::CEnumType {
        const NORMAL = cef_ssl_content_status_t::SSL_CONTENT_NORMAL_CONTENT.0;
        const DISPLAYED_INSECURE = cef_ssl_content_status_t::SSL_CONTENT_DISPLAYED_INSECURE_CONTENT.0;
        const RAN_INSECURE = cef_ssl_content_status_t::SSL_CONTENT_RAN_INSECURE_CONTENT.0;
    }
}

impl CertStatus {
    pub fn is_cert_status_error(&self) -> bool {
        unsafe { cef_is_cert_status_error(cef_cert_status_t(self.bits())) != 0 }
    }
}

ref_counted_ptr! {
    /// Structure representing SSL information.
    pub struct SSLInfo(*mut _cef_sslinfo_t);
}

ref_counted_ptr!{
    pub struct SSLStatus(*mut cef_sslstatus_t);
}

impl SSLInfo {
    /// Returns a set containing any and all problems verifying the server
    /// certificate.
    pub fn get_cert_status(&self) -> CertStatus {
        self.0
            .get_cert_status
            .map(|get_cert_status| CertStatus::from_bits_truncate(unsafe { get_cert_status(self.0.as_ptr()).0 }))
            .unwrap_or_default()
    }
    /// Returns the X.509 certificate.
    pub fn get_x509certificate(&self) -> X509Certificate {
        let get_x509certificate = self.0.get_x509certificate.unwrap();
        unsafe { X509Certificate::from_ptr_unchecked(get_x509certificate(self.0.as_ptr())) }
    }
}

impl SSLStatus {
    pub fn is_secure_connection(&self) -> bool {
        unsafe{ self.0.is_secure_connection.unwrap()(self.as_ptr()) != 0 }
    }
    pub fn get_cert_status(&self) -> CertStatus {
        unsafe{ CertStatus::from_bits_truncate(self.0.get_cert_status.unwrap()(self.as_ptr()).0) }
    }
    pub fn get_ssl_version(&self) -> SSLVersion {
        unsafe{ SSLVersion::from_unchecked(self.0.get_sslversion.unwrap()(self.as_ptr())) }
    }
    pub fn get_content_status(&self) -> ContentStatus {
        unsafe{ ContentStatus::from_bits_truncate(self.0.get_content_status.unwrap()(self.as_ptr()).0) }
    }
    pub fn get_x509certificate(&self) -> X509Certificate {
        unsafe{ X509Certificate::from_ptr_unchecked(self.0.get_x509certificate.unwrap()(self.as_ptr())) }
    }
}
