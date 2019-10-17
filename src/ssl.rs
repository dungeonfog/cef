use cef_sys::{_cef_sslinfo_t, cef_cert_status_t, cef_is_cert_status_error, _cef_x509certificate_t};
use std::collections::HashSet;

/// Supported certificate status code values. See net\cert\cert_status_flags.h
/// for more information. CERT_STATUS_NONE is new in CEF because we use an
/// enum while cert_status_flags.h uses a typedef and static const variables.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CertStatus {
    None = cef_cert_status_t::CERT_STATUS_NONE.0,
    CommonNameInvalid = cef_cert_status_t::CERT_STATUS_COMMON_NAME_INVALID.0,
    DateInvalid = cef_cert_status_t::CERT_STATUS_DATE_INVALID.0,
    AuthorityInvalid = cef_cert_status_t::CERT_STATUS_AUTHORITY_INVALID.0,
    NoRevocationMechanism = cef_cert_status_t::CERT_STATUS_NO_REVOCATION_MECHANISM.0,
    UnableToCheckRevocation = cef_cert_status_t::CERT_STATUS_UNABLE_TO_CHECK_REVOCATION.0,
    Revoked = cef_cert_status_t::CERT_STATUS_REVOKED.0,
    Invalid = cef_cert_status_t::CERT_STATUS_INVALID.0,
    WeakSignatureAlgorithm = cef_cert_status_t::CERT_STATUS_WEAK_SIGNATURE_ALGORITHM.0,
    NonUniqueName = cef_cert_status_t::CERT_STATUS_NON_UNIQUE_NAME.0,
    WeakKey = cef_cert_status_t::CERT_STATUS_WEAK_KEY.0,
    PinnedKeyMissing = cef_cert_status_t::CERT_STATUS_PINNED_KEY_MISSING.0,
    NameConstraintViolation = cef_cert_status_t::CERT_STATUS_NAME_CONSTRAINT_VIOLATION.0,
    ValidityTooLong = cef_cert_status_t::CERT_STATUS_VALIDITY_TOO_LONG.0,
    IsEV = cef_cert_status_t::CERT_STATUS_IS_EV.0,
    RevCheckingEnabled = cef_cert_status_t::CERT_STATUS_REV_CHECKING_ENABLED.0,
    Sha1SignaturePresent = cef_cert_status_t::CERT_STATUS_SHA1_SIGNATURE_PRESENT.0,
    CTComplianceFailed = cef_cert_status_t::CERT_STATUS_CT_COMPLIANCE_FAILED.0,
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

// impl HashSet<CertStatus> {
//     pub fn is_cert_status_error(&self) -> bool {
//         unsafe { cef_is_cert_status_error(CertStatus::as_mask(self)) != 0 }
//     }
// }

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
