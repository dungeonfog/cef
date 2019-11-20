use crate::{
    string::{CefString, CefStringList},
    values::BinaryValue,
};
use cef_sys::{cef_x509certificate_t, cef_x509cert_principal_t};
use chrono::{DateTime, Utc};

ref_counted_ptr! {
    /// Structure representing a X.509 certificate.
    pub struct X509Certificate(*mut cef_x509certificate_t);
}

ref_counted_ptr! {
    /// Structure representing the issuer or subject field of an X.509 certificate.
    pub struct X509CertPrincipal(*mut cef_x509cert_principal_t);
}

impl X509Certificate {
    /// Returns the subject of the X.509 certificate. For HTTPS server certificates
    /// this represents the web server.  The common name of the subject should
    /// match the host name of the web server.
    pub fn get_subject(&self) -> X509CertPrincipal {
        unsafe {
            X509CertPrincipal::from_ptr_unchecked(self.0.get_subject.unwrap()(self.as_ptr()))
        }
    }
    /// Returns the issuer of the X.509 certificate.
    pub fn get_issuer(&self) -> X509CertPrincipal {
        unsafe {
            X509CertPrincipal::from_ptr_unchecked(self.0.get_issuer.unwrap()(self.as_ptr()))
        }
    }
    /// Returns the DER encoded serial number for the X.509 certificate. The value
    /// possibly includes a leading 00 byte.
    pub fn get_serial_number(&self) -> BinaryValue {
        unsafe {
            BinaryValue::from_ptr_unchecked(self.0.get_serial_number.unwrap()(self.as_ptr()))
        }
    }
    /// Returns the date before which the X.509 certificate is invalid.
    /// CefTime.GetTimeT() will return 0 if no date was specified.
    pub fn get_valid_start(&self) -> DateTime<Utc> {
        crate::values::cef_time_to_date_time(unsafe {
            self.0.get_valid_start.unwrap()(self.as_ptr())
        })
    }
    /// Returns the date after which the X.509 certificate is invalid.
    /// CefTime.GetTimeT() will return 0 if no date was specified.
    pub fn get_valid_expiry(&self) -> DateTime<Utc> {
        crate::values::cef_time_to_date_time(unsafe {
            self.0.get_valid_expiry.unwrap()(self.as_ptr())
        })
    }
    /// Returns the DER encoded data for the X.509 certificate.
    pub fn get_derencoded(&self) -> BinaryValue {
        unsafe {
            BinaryValue::from_ptr_unchecked(self.0.get_derencoded.unwrap()(self.as_ptr()))
        }
    }
    /// Returns the PEM encoded data for the X.509 certificate.
    pub fn get_pemencoded(&self) -> BinaryValue {
        unsafe {
            BinaryValue::from_ptr_unchecked(self.0.get_pemencoded.unwrap()(self.as_ptr()))
        }
    }
    /// Returns the number of certificates in the issuer chain. If 0, the
    /// certificate is self-signed.
    pub fn get_issuer_chain_size(&self) -> usize {
        unsafe{ self.0.get_issuer_chain_size.unwrap()(self.as_ptr()) }
    }
    /// Returns the DER encoded data for the certificate issuer chain. If we failed
    /// to encode a certificate in the chain it is still present in the array but
    /// is an `None` string.
    pub fn get_der_encoded_issuer_chain(&self, chain: &mut Vec<BinaryValue>) {
        // the CEF C API seems basically unusable for this. I've got no idea how to implement it.
        unimplemented!()
    }
    /// Returns the PEM encoded data for the certificate issuer chain. If we failed
    /// to encode a certificate in the chain it is still present in the array but
    /// is an `None` string.
    pub fn get_pem_encoded_issuer_chain(&self, chain: &mut Vec<BinaryValue>) {
        // the CEF C API seems basically unusable for this. I've got no idea how to implement it.
        unimplemented!()
    }
}

impl X509CertPrincipal {
    /// Returns a name that can be used to represent the issuer. It tries in this
    /// order: Common Name (CN), Organization Name (O) and Organizational Unit Name
    /// (OU) and returns the first non-NULL one found.
    pub fn get_display_name(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_display_name.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the common name.
    pub fn get_common_name(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_common_name.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the locality name.
    pub fn get_locality_name(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_locality_name.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the state or province name.
    pub fn get_state_or_province_name(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_state_or_province_name.unwrap()(self.as_ptr())).into()
        }
    }
    /// Returns the country name.
    pub fn get_country_name(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(self.0.get_country_name.unwrap()(self.as_ptr())).into()
        }
    }
    /// Retrieve the list of street addresses.
    pub fn get_street_addresses(&self, addresses: &mut Vec<String>) {
        let mut list = CefStringList::new();
        unsafe {
            self.0.get_street_addresses.unwrap()(self.as_ptr(), list.as_mut_ptr());
        }
        addresses.extend(list.into_iter().map(|s| String::from(s)));
    }
    /// Retrieve the list of organization names.
    pub fn get_organization_names(&self, names: &mut Vec<String>) {
        let mut list = CefStringList::new();
        unsafe {
            self.0.get_organization_names.unwrap()(self.as_ptr(), list.as_mut_ptr());
        }
        names.extend(list.into_iter().map(|s| String::from(s)));
    }
    /// Retrieve the list of organization unit names.
    pub fn get_organization_unit_names(&self, names: &mut Vec<String>) {
        let mut list = CefStringList::new();
        unsafe {
            self.0.get_organization_unit_names.unwrap()(self.as_ptr(), list.as_mut_ptr());
        }
        names.extend(list.into_iter().map(|s| String::from(s)));
    }
    /// Retrieve the list of domain components.
    pub fn get_domain_components(&self, components: &mut Vec<String>) {
        let mut list = CefStringList::new();
        unsafe {
            self.0.get_domain_components.unwrap()(self.as_ptr(), list.as_mut_ptr());
        }
        components.extend(list.into_iter().map(|s| String::from(s)));
    }
}
