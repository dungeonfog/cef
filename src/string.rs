use cef_sys::{cef_string_t, cef_string_utf8_to_utf16, cef_string_list_t, cef_string_list_alloc, cef_string_list_size, cef_string_list_value, cef_string_list_free};

#[derive(Default)]
pub(crate) struct CefString(cef_string_t);

impl CefString {
    pub fn new(source: &str) -> Self {
        let mut instance = cef_string_t::default();
        let len = source.len();
        unsafe {
            cef_string_utf8_to_utf16(source.as_ptr() as *const std::os::raw::c_char, len, &mut instance);
        }
        CefString(instance)
    }
    pub fn copy_raw_to_string(source: *const cef_string_t) -> Option<String> {
        if source.is_null() {
            None
        } else {
            Some(String::from_utf16_lossy(unsafe { std::slice::from_raw_parts((*source).str, (*source).length) }))
        }
    }
}

impl Drop for CefString {
    fn drop(&mut self) {
        if let Some(dtor) = self.0.dtor {
            unsafe { dtor(&mut self.0 as *mut cef_string_t as *mut u16); }
        }
    }
}

impl std::convert::AsRef<cef_string_t> for CefString {
    fn as_ref(&self) -> &cef_string_t {
        &self.0
    }
}

impl Into<String> for CefString {
    fn into(self) -> String {
        Self::copy_raw_to_string(&self.0).unwrap()
    }
}

pub(crate) struct CefStringList(cef_string_list_t);

impl Default for CefStringList {
    fn default() -> Self {
        Self(unsafe { cef_string_list_alloc() })
    }
}

impl Drop for CefStringList {
    fn drop(&mut self) {
        unsafe { cef_string_list_free(self.0); }
    }
}

impl Into<cef_string_list_t> for CefStringList {
    fn into(self) -> cef_string_list_t {
        self.0
    }
}

impl Into<Vec<String>> for CefStringList {
    fn into(self) -> Vec<String> {
        from_string_list(self.0)
    }
}

impl CefStringList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self) -> cef_string_list_t {
        self.0
    }
}

pub(crate) fn from_string_list(list: cef_string_list_t) -> Vec<String> {
    (0..unsafe { cef_string_list_size(list) }).map(|index| {
        let item = CefString::default();
        unsafe {
            cef_string_list_value(list, index, item.as_ref() as *const cef_string_t as *mut cef_string_t);
        }
        item.into()
    }).collect()
}
