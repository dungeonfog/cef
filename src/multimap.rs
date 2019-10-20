use cef_sys::{
    cef_string_multimap_alloc, cef_string_multimap_append, cef_string_multimap_clear,
    cef_string_multimap_enumerate, cef_string_multimap_find_count, cef_string_multimap_free,
    cef_string_multimap_key, cef_string_multimap_size, cef_string_multimap_t,
    cef_string_multimap_value,
};
use std::collections::HashMap;

use crate::string::CefString;

/// CEF string multimaps are a set of key/value string pairs.
/// More than one value can be assigned to a single key.
pub(crate) struct MultiMap(cef_string_multimap_t);

impl MultiMap {
    /// Allocate a new string multimap.
    pub(crate) fn new() -> Self {
        Self(unsafe { cef_string_multimap_alloc() })
    }
    pub(crate) fn as_ptr(&self) -> cef_string_multimap_t {
        self.0
    }
    /// Return the number of elements in the string multimap.
    pub(crate) fn len(&self) -> usize {
        unsafe { cef_string_multimap_size(self.0) }
    }
    /// Return the number of values with the specified key.
    pub(crate) fn find_count(&self, key: &str) -> usize {
        unsafe { cef_string_multimap_find_count(self.0, CefString::new(key).as_ptr()) }
    }
    /// Return the value_index-th value with the specified key.
    pub(crate) fn enumerate(&self, key: &str, value_index: usize) -> Result<String, ()> {
        let mut result = unsafe { std::mem::zeroed() };
        if unsafe {
            cef_string_multimap_enumerate(
                self.0,
                CefString::new(key).as_ptr(),
                value_index,
                &mut result,
            ) == 1
        } {
            Ok(CefString::from(result).into())
        } else {
            Err(())
        }
    }
    /// Return the key at the specified zero-based string multimap index.
    pub(crate) fn get_key(&self, index: usize) -> Result<String, ()> {
        let mut result = unsafe { std::mem::zeroed() };
        if unsafe { cef_string_multimap_key(self.0, index, &mut result) == 1 } {
            Ok(CefString::from(result).into())
        } else {
            Err(())
        }
    }
    /// Return the value at the specified zero-based string multimap index.
    pub(crate) fn get_value(&self, index: usize) -> Result<String, ()> {
        let mut result = unsafe { std::mem::zeroed() };
        if unsafe { cef_string_multimap_value(self.0, index, &mut result) == 1 } {
            Ok(CefString::from(result).into())
        } else {
            Err(())
        }
    }
    /// Append a new key/value pair at the end of the string multimap.
    pub(crate) fn append(&mut self, key: &str, value: &str) -> Result<(), ()> {
        if unsafe {
            cef_string_multimap_append(
                self.0,
                CefString::new(key).as_ptr(),
                CefString::new(value).as_ptr(),
            ) == 1
        } {
            Ok(())
        } else {
            Err(())
        }
    }
    /// Clear the string multimap.
    pub(crate) fn clear(&mut self) {
        unsafe {
            cef_string_multimap_clear(self.0);
        }
    }
}

impl Drop for MultiMap {
    /// Free the string multimap.
    fn drop(&mut self) {
        unsafe {
            cef_string_multimap_free(self.0);
        }
    }
}

#[doc(hidden)]
impl From<cef_string_multimap_t> for MultiMap {
    fn from(map: cef_string_multimap_t) -> Self {
        Self(map)
    }
}

#[doc(hidden)]
impl From<&HashMap<String, Vec<String>>> for MultiMap {
    fn from(map: &HashMap<String, Vec<String>>) -> Self {
        let mut result = MultiMap::new();

        for (key, list) in map.iter() {
            for value in list.iter() {
                result.append(&key, &value).ok();
            }
        }

        result
    }
}

impl Into<HashMap<String, Vec<String>>> for MultiMap {
    fn into(self) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        for idx in 0..self.len() {
            if let (Ok(key), Ok(value)) = (self.get_key(idx), self.get_value(idx)) {
                result.entry(key).or_insert_with(Vec::new).push(value);
            }
        }

        result
    }
}
