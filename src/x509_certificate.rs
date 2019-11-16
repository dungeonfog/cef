use cef_sys::{cef_x509certificate_t, cef_x509cert_principal_t};

ref_counted_ptr! {
    // TODO: IMPLEMENT
    pub struct X509Certificate(*mut cef_x509certificate_t);
}

ref_counted_ptr! {
    // TODO: IMPLEMENT
    pub struct X509CertPrincipal(*mut cef_x509cert_principal_t);
}
