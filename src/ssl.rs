use cef_sys::{_cef_sslinfo_t, cef_cert_status_t, cef_is_cert_status_error, _cef_x509certificate_t};
use std::collections::HashSet;

/// Supported certificate status code values. See net\cert\cert_status_flags.h
/// for more information. CERT_STATUS_NONE is new in CEF because we use an
/// enum while cert_status_flags.h uses a typedef and static const variables.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CertStatus {
    None = cef_cert_status_t::CERT_STATUS_NONE,
    CommonNameInvalid = cef_cert_status_t::CERT_STATUS_COMMON_NAME_INVALID,
    DateInvalid = cef_cert_status_t::CERT_STATUS_DATE_INVALID,
    AuthorityInvalid = cef_cert_status_t::CERT_STATUS_AUTHORITY_INVALID,
    NoRevocationMechanism = cef_cert_status_t::CERT_STATUS_NO_REVOCATION_MECHANISM,
    UnableToCheckRevocation = cef_cert_status_t::CERT_STATUS_UNABLE_TO_CHECK_REVOCATION,
    Revoked = cef_cert_status_t::CERT_STATUS_REVOKED,
    Invalid = cef_cert_status_t::CERT_STATUS_INVALID,
    WeakSignatureAlgorithm = cef_cert_status_t::CERT_STATUS_WEAK_SIGNATURE_ALGORITHM,
    NonUniqueName = cef_cert_status_t::CERT_STATUS_NON_UNIQUE_NAME,
    WeakKey = cef_cert_status_t::CERT_STATUS_WEAK_KEY,
    PinnedKeyMissing = cef_cert_status_t::CERT_STATUS_PINNED_KEY_MISSING,
    NameConstraintViolation = cef_cert_status_t::CERT_STATUS_NAME_CONSTRAINT_VIOLATION,
    ValidityTooLong = cef_cert_status_t::CERT_STATUS_VALIDITY_TOO_LONG,
    IsEV = cef_cert_status_t::CERT_STATUS_IS_EV,
    RevCheckingEnabled = cef_cert_status_t::CERT_STATUS_REV_CHECKING_ENABLED,
    Sha1SignaturePresent = cef_cert_status_t::CERT_STATUS_SHA1_SIGNATURE_PRESENT,
    CTComplianceFailed = cef_cert_status_t::CERT_STATUS_CT_COMPLIANCE_FAILED,
}

impl CertStatus {
    pub(crate) fn as_vec(status: cef_cert_status_t) -> HashSet<CertStatus> {
        [
            CertStatus::CommonNameInvalid,
            CertStatus::DateInvalid,
            CertStatus::AuthorityInvalid,
            CertStatus::NoRevocationMechanism,
            CertStatus::UnableToCheckRevocation,
            CertStatus::Revoked,
            CertStatus::Invalid,
            CertStatus::WeakSignatureAlgorithm,
            CertStatus::NonUniqueName,
            CertStatus::WeakKey,
            CertStatus::PinnedKeyMissing,
            CertStatus::NameConstraintViolation,
            CertStatus::ValidityTooLong,
            CertStatus::IsEv,
            CertStatus::RevCheckingEnabled,
            CertStatus::Sha1SignaturePresent,
            CertStatus::CtComplianceFailed,
        ]
        .iter()
        .filter(|flag| ((**flag) as u32 & status) != 0)
        .cloned()
        .collect()
    }

    pub(crate) fn as_mask<'a, I: 'a + Iterator<Item = &'a Self>>(status_flags: I) -> cef_cert_status_t {
        cef_cert_status_t(status_flags.fold(0, |mask, flag| mask | (*flag as i32)))
    }
}

impl HashSet<CertStatus> {
    pub fn is_cert_status_error(&self) -> bool {
        unsafe { cef_is_cert_status_error(CertStatus::as_mask(self)) != 0 }
    }
}

ref_counted_ptr! {
    /// Structure representing SSL information.
    pub struct SSLInfo(*mut _cef_sslinfo_t);
}

impl SSLInfo {
    /// Returns a set containing any and all problems verifying the server
    /// certificate.
    pub fn get_cert_status(&self) -> HashSet<CertStatus> {
        self.0.get_cert_status.map(|get_cert_status| {
            CertStatus::as_vec(unsafe { get_cert_status(self.0.as_ptr()) })
        }).unwrap_or_default()
    }
    /// Returns the X.509 certificate.
    pub fn get_x509certificate(&self) -> X509Certificate {
        let get_x509certificate = self.0.get_x509certificate.unwrap();
        unsafe { X509Certificate::from_raw_unchecked(get_x509certificate(self.0.as_ptr())) }
    }
}


ref_counted_ptr! {
    pub struct X509Certificate(*mut _cef_x509certificate_t);
}

impl X509Certificate {
    pub(crate) fn as_ptr(&self) -> *const _cef_x509certificate_t {
        &self.0
    }
    pub(crate) fn as_ptr_mut(&mut self) -> *mut _cef_x509certificate_t {
        &mut self.0
    }
}
