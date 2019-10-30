pub mod iter;
use self::iter::DictionaryValueKeysIter;
use cef_sys::{
    cef_binary_value_create, cef_binary_value_t, cef_dictionary_value_create,
    cef_dictionary_value_t, cef_list_value_create, cef_list_value_t, cef_point_t, cef_range_t,
    cef_size_t, cef_string_userfree_utf16_free, cef_value_create, cef_value_t, cef_value_type_t,
    cef_rect_t,
};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt,
    marker::PhantomData,
};

use crate::string::{CefString, CefStringList, CefStringListIntoIter};

#[derive(Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum ValueType {
    Invalid = cef_value_type_t::VTYPE_INVALID as i32,
    Null = cef_value_type_t::VTYPE_NULL as i32,
    Bool = cef_value_type_t::VTYPE_BOOL as i32,
    Int = cef_value_type_t::VTYPE_INT as i32,
    Double = cef_value_type_t::VTYPE_DOUBLE as i32,
    String = cef_value_type_t::VTYPE_STRING as i32,
    Binary = cef_value_type_t::VTYPE_BINARY as i32,
    Dictionary = cef_value_type_t::VTYPE_DICTIONARY as i32,
    List = cef_value_type_t::VTYPE_LIST as i32,
}

#[derive(Debug, Clone)]
pub enum StoredValue {
    Invalid,
    Null,
    Bool(bool),
    Int(i32),
    Double(f64),
    String(String),
    Binary(BinaryValue),
    Dictionary(DictionaryValue),
    List(ListValue),
}

ref_counted_ptr! {
    pub(crate) struct Value(*mut cef_value_t);
}

impl Value {
    pub(crate) fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_value_create()) }
    }
    /// Returns true if the underlying data is valid. This will always be true
    /// for simple types. For complex types (binary, dictionary and list) the
    /// underlying data may become invalid if owned by another object (e.g. list or
    /// dictionary) and that other object is then modified or destroyed. This value
    /// object can be re-used by calling `set_*()` even if the underlying data is
    /// invalid.
    pub(crate) fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub(crate) fn is_owned(&self) -> bool {
        self.0
            .is_owned
            .map(|is_owned| unsafe { is_owned(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the underlying data is read-only. Some APIs may expose
    /// read-only objects.
    pub(crate) fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) != 0 })
            .unwrap_or(true)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data. If true modifications to this object will also affect `that`
    /// object and vice-versa.
    pub(crate) fn is_same(&self, that: &Value) -> bool {
        self.0
            .is_same
            .map(|is_same| unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the underlying value type.
    pub(crate) fn get_type(&self) -> ValueType {
        self.0
            .get_type
            .map(|get_type| match unsafe { get_type(self.as_ptr()) } {
                cef_value_type_t::VTYPE_NULL => ValueType::Null,
                cef_value_type_t::VTYPE_BOOL => ValueType::Bool,
                cef_value_type_t::VTYPE_INT => ValueType::Int,
                cef_value_type_t::VTYPE_DOUBLE => ValueType::Double,
                cef_value_type_t::VTYPE_STRING => ValueType::String,
                cef_value_type_t::VTYPE_BINARY => ValueType::Binary,
                cef_value_type_t::VTYPE_DICTIONARY => ValueType::Dictionary,
                cef_value_type_t::VTYPE_LIST => ValueType::List,
                _ => ValueType::Invalid,
            })
            .unwrap_or(ValueType::Invalid)
    }
    /// Returns the underlying value as type bool.
    pub(crate) fn to_bool(&self) -> bool {
        self.0
            .get_bool
            .map(|get_bool| unsafe { get_bool(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the underlying value as type int.
    pub(crate) fn to_int(&self) -> i32 {
        self.0
            .get_int
            .map(|get_int| unsafe { get_int(self.as_ptr()) as i32 })
            .unwrap_or(0)
    }
    /// Returns the underlying value as type double.
    pub(crate) fn to_double(&self) -> f64 {
        self.0
            .get_double
            .map(|get_double| unsafe { get_double(self.as_ptr()) })
            .unwrap_or(0.0)
    }
    /// Returns the underlying value as type string.
    pub(crate) fn to_string(&self) -> String {
        self.0
            .get_string
            .and_then(|get_string| {
                let s = unsafe { get_string(self.as_ptr()) };
                let result = unsafe { CefString::from_ptr(s).map(String::from) };
                unsafe {
                    cef_string_userfree_utf16_free(s as *mut _);
                }
                result
            })
            .unwrap_or_else(String::new)
    }
    /// Returns the underlying value as type binary. The returned reference may
    /// become invalid if the value is owned by another object or if ownership is
    /// transferred to another object in the future. To maintain a reference to the
    /// value after assigning ownership to a dictionary or list pass this object to
    /// the [set_value()] function instead of passing the returned reference to
    /// [set_binary()].
    pub(crate) fn try_to_binary(&self) -> Option<BinaryValue> {
        self.0
            .get_binary
            .and_then(|get_binary| unsafe { BinaryValue::from_ptr(get_binary(self.as_ptr())) })
    }
    /// Returns the underlying value as type dictionary. The returned reference may
    /// become invalid if the value is owned by another object or if ownership is
    /// transferred to another object in the future. To maintain a reference to the
    /// value after assigning ownership to a dictionary or list pass this object to
    /// the [set_value()] function instead of passing the returned reference to
    /// [set_dictionary()].
    pub(crate) fn try_to_dictionary(&self) -> Option<DictionaryValue> {
        self.0.get_dictionary.and_then(|get_dictionary| unsafe {
            DictionaryValue::from_ptr(get_dictionary(self.as_ptr()))
        })
    }
    /// Returns the underlying value as type list. The returned reference may
    /// become invalid if the value is owned by another object or if ownership is
    /// transferred to another object in the future. To maintain a reference to the
    /// value after assigning ownership to a dictionary or list pass this object to
    /// the [set_value()] function instead of passing the returned reference to
    /// [set_list()].
    pub(crate) fn try_to_list(&self) -> Option<ListValue> {
        self.0
            .get_list
            .and_then(|get_list| unsafe { ListValue::from_ptr(get_list(self.as_ptr())) })
    }
    /// Sets the underlying value as type null. Returns true if the value was
    /// set successfully.
    pub(crate) fn set_null(&self) -> bool {
        self.0
            .set_null
            .map(|set_null| unsafe { set_null(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Sets the underlying value as type bool. Returns true if the value was
    /// set successfully.
    pub(crate) fn set_bool(&self, value: bool) -> bool {
        self.0
            .set_bool
            .map(|set_bool| unsafe { set_bool(self.as_ptr(), if value { 1 } else { 0 }) != 0 })
            .unwrap_or(false)
    }
    /// Sets the underlying value as type int. Returns true if the value was
    /// set successfully.
    pub(crate) fn set_int(&self, value: i32) -> bool {
        self.0
            .set_int
            .map(|set_int| unsafe { set_int(self.as_ptr(), value as std::os::raw::c_int) != 0 })
            .unwrap_or(false)
    }
    /// Sets the underlying value as type double. Returns true if the value was
    /// set successfully.
    pub(crate) fn set_double(&self, value: f64) -> bool {
        self.0
            .set_double
            .map(|set_double| unsafe { set_double(self.as_ptr(), value) != 0 })
            .unwrap_or(false)
    }
    /// Sets the underlying value as type string. Returns true if the value was
    /// set successfully.
    pub(crate) fn set_string(&self, value: &str) -> bool {
        self.0
            .set_string
            .map(|set_string| unsafe {
                set_string(self.as_ptr(), CefString::new(value).as_ptr()) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the underlying value as type binary. Returns true if the value was
    /// set successfully. This object keeps a reference to |value| and ownership of
    /// the underlying data remains unchanged.
    pub(crate) fn set_binary(&self, value: BinaryValue) -> bool {
        self.0
            .set_binary
            .map(|set_binary| unsafe { set_binary(self.as_ptr(), value.into_raw()) != 0 })
            .unwrap_or(false)
    }
    /// Sets the underlying value as type dict. Returns true if the value was
    /// set successfully. This object keeps a reference to `value` and ownership of
    /// the underlying data remains unchanged.
    pub(crate) fn set_dictionary(&self, value: DictionaryValue) -> bool {
        self.0
            .set_dictionary
            .map(|set_dictionary| unsafe { set_dictionary(self.as_ptr(), value.into_raw()) != 0 })
            .unwrap_or(false)
    }
    /// Sets the underlying value as type list. Returns true if the value was
    /// set successfully. This object keeps a reference to `value` and ownership of
    /// the underlying data remains unchanged.
    pub(crate) fn set_list(&self, value: ListValue) -> bool {
        self.0
            .set_list
            .map(|set_list| unsafe { set_list(self.as_ptr(), value.into_raw()) != 0 })
            .unwrap_or(false)
    }
}

impl PartialEq for Value {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.0
            .is_equal
            .map(|is_equal| unsafe { is_equal(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
}

impl crate::cef_helper_traits::DeepClone for Value {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn deep_clone(&self) -> Self {
        unsafe { Self::from_ptr_unchecked((self.0.copy.unwrap())(self.as_ptr())) }
    }
}

impl From<Value> for StoredValue {
    fn from(value: Value) -> StoredValue {
        match value.get_type() {
            ValueType::Invalid => StoredValue::Invalid,
            ValueType::Null => StoredValue::Null,
            ValueType::Bool => StoredValue::Bool(value.to_bool()),
            ValueType::Int => StoredValue::Int(value.to_int()),
            ValueType::Double => StoredValue::Double(value.to_double()),
            ValueType::String => StoredValue::String(value.to_string()),
            ValueType::Binary => {
                let binary = value.try_to_binary().unwrap();
                StoredValue::Binary(binary)
            }
            ValueType::Dictionary => {
                let dictionary = value.try_to_dictionary().unwrap();
                StoredValue::Dictionary(dictionary)
            }
            ValueType::List => {
                let list = value.try_to_list().unwrap();
                StoredValue::List(list)
            }
        }
    }
}

impl TryFrom<StoredValue> for Value {
    type Error = &'static str;

    fn try_from(sv: StoredValue) -> Result<Self, Self::Error> {
        let value = Value::new();
        if match sv {
            StoredValue::Invalid | StoredValue::Null => true,
            StoredValue::Bool(b) => value.set_bool(b),
            StoredValue::Int(i) => value.set_int(i),
            StoredValue::Double(f) => value.set_double(f),
            StoredValue::String(s) => value.set_string(&s),
            StoredValue::Binary(b) => value.set_binary(b),
            StoredValue::Dictionary(d) => value.set_dictionary(d),
            StoredValue::List(l) => value.set_list(l),
        } {
            Ok(value)
        } else {
            Err("Unable to create type")
        }
    }
}

ref_counted_ptr! {
    // #[derive(Eq)]
    pub struct BinaryValue(*mut cef_binary_value_t);
}

impl BinaryValue {
    /// Creates a new object that is not owned by any other object. The specified
    /// `data` will be copied.
    pub fn new(data: &[u8]) -> Self {
        unsafe {
            Self::from_ptr_unchecked(cef_binary_value_create(
                data.as_ptr() as *const std::os::raw::c_void,
                data.len(),
            ))
        }
    }
    /// Returns true if this object is valid. This object may become invalid if
    /// the underlying data is owned by another object (e.g. list or dictionary)
    /// and that other object is then modified or destroyed. Do not call any other
    /// functions if this function returns false.
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub fn is_owned(&self) -> bool {
        self.0
            .is_owned
            .map(|is_owned| unsafe { is_owned(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data.
    pub fn is_same(&self, that: &BinaryValue) -> bool {
        self.0
            .is_same
            .map(|is_same| unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the data size.
    pub fn len(&self) -> usize {
        self.0
            .get_size
            .map(|get_size| unsafe { get_size(self.as_ptr()) })
            .unwrap_or(0)
    }
    pub fn push_to_vec(&self, vec: &mut Vec<u8>) {
        let len = self.len();
        let target_vec_len = vec.len() + len;
        let reserve_additional = target_vec_len.checked_sub(vec.capacity());
        if let Some(reserve) = reserve_additional {
            vec.reserve(reserve);
        }

        assert!(vec.capacity() >= vec.len() + len);
        let ptr = vec.as_mut_ptr();
        unsafe {
            let get_data = self.0.get_data.unwrap();
            let bytes = get_data(
                self.as_ptr(),
                ptr.offset(vec.len().try_into().unwrap()) as *mut std::ffi::c_void,
                vec.capacity() - vec.len(),
                0,
            );
            let new_len = vec.len() + bytes;
            assert!(vec.capacity() >= new_len);
            vec.set_len(new_len)
        }
    }
    pub(crate) fn to_vec(&self) -> Vec<u8> {
        self.0
            .get_data
            .map(|get_data| {
                let len = self.len();
                let mut result = vec![0; len];
                let out_len = unsafe {
                    get_data(
                        self.as_ptr(),
                        result.as_mut_ptr() as *mut std::ffi::c_void,
                        len,
                        0,
                    )
                };
                result.truncate(out_len);
                result
            })
            .unwrap_or_default()
    }
}

impl From<BinaryValue> for Vec<u8> {
    fn from(value: BinaryValue) -> Vec<u8> {
        value.to_vec()
    }
}

impl fmt::Debug for BinaryValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut vec = Vec::new();
        self.push_to_vec(&mut vec);
        fmt::Debug::fmt(&vec, f)
    }
}

// TODO: CREATE `BinaryValueCursor`
// impl Read for BinaryValue {
//     fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
//         self.as_ref()
//             .get_data
//             .and_then(|get_data| {
//                 Some(unsafe {
//                     get_data(
//                         self.0,
//                         buf.as_mut_ptr() as *mut std::ffi::c_void,
//                         buf.len(),
//                         self.1,
//                     )
//                 })
//             })
//             .and_then(|result| {
//                 self.1 += result;
//                 Some(result)
//             })
//             .ok_or_else(|| {
//                 std::io::Error::new(
//                     std::io::ErrorKind::InvalidData,
//                     "cef_binary_value_t is invalid",
//                 )
//             })
//     }
// }

// impl std::io::Seek for BinaryValue {
//     fn seek(&self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
//         self.1 = match pos {
//             std::io::SeekFrom::Start(offset) => usize::try_from(offset),
//             std::io::SeekFrom::Current(offset) => usize::try_from(self.1 as i64 + offset),
//             std::io::SeekFrom::End(offset) => {
//                 if offset > self.1 as i64 {
//                     Ok(0)
//                 } else {
//                     Ok((self.1 as i64 - offset) as usize)
//                 }
//             }
//         }
//         .map_err(|err| {
//             std::io::Error::new(std::io::ErrorKind::InvalidInput, "offset is out of range")
//         })?;
//         Ok(self.1 as u64)
//     }
// }

impl PartialEq for BinaryValue {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.0
            .is_equal
            .map(|is_equal| unsafe { is_equal(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
}

impl crate::cef_helper_traits::DeepClone for BinaryValue {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn deep_clone(&self) -> Self {
        unsafe { Self::from_ptr_unchecked((self.0.copy.unwrap())(self.as_ptr())) }
    }
}

ref_counted_ptr! {
    pub struct DictionaryValue(*mut cef_dictionary_value_t);
}

impl DictionaryValue {
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_dictionary_value_create()) }
    }
    /// Returns true if this object is valid. This object may become invalid if
    /// the underlying data is owned by another object (e.g. list or dictionary)
    /// and that other object is then modified or destroyed. Do not call any other
    /// functions if this function returns false.
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub fn is_owned(&self) -> bool {
        self.0
            .is_owned
            .map(|is_owned| unsafe { is_owned(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the underlying data is read-only. Some APIs may expose
    /// read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) != 0 })
            .unwrap_or(true)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data.
    pub fn is_same(&self, that: &DictionaryValue) -> bool {
        self.0
            .is_same
            .map(|is_same| unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.0
            .get_size
            .map(|get_size| unsafe { get_size(self.as_ptr()) })
            .unwrap_or(0)
    }
    /// Removes all values. Returns true on success.
    pub fn clear(&self) -> bool {
        self.0
            .clear
            .map(|clear| unsafe { clear(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the current dictionary has a value for the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.0
            .has_key
            .map(|has_key| unsafe { has_key(self.as_ptr(), CefString::new(key).as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Reads all keys for this dictionary into the specified vector.
    pub fn keys(&self) -> DictionaryValueKeysIter {
        let list = self
            .0
            .get_keys
            .and_then(|get_keys| {
                let mut list = CefStringList::new();
                if unsafe { get_keys(self.as_ptr(), list.as_mut_ptr()) } != 0 {
                    Some(list)
                } else {
                    None
                }
            })
            .unwrap_or_else(CefStringList::new);
        DictionaryValueKeysIter {
            keys: list.into_iter(),
            _dictionary: PhantomData,
        }
    }
    /// Removes the value at the specified key. Returns true if the value
    /// is removed successfully.
    pub fn remove(&self, key: &str) -> bool {
        self.0
            .remove
            .map(|remove| unsafe { remove(self.as_ptr(), CefString::new(key).as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the value type for the specified key.
    pub(crate) fn get_type(&self, key: &str) -> ValueType {
        self.0
            .get_type
            .map(|get_type| {
                match unsafe { get_type(self.as_ptr(), CefString::new(key).as_ptr()) } {
                    cef_value_type_t::VTYPE_NULL => ValueType::Null,
                    cef_value_type_t::VTYPE_BOOL => ValueType::Bool,
                    cef_value_type_t::VTYPE_INT => ValueType::Int,
                    cef_value_type_t::VTYPE_DOUBLE => ValueType::Double,
                    cef_value_type_t::VTYPE_STRING => ValueType::String,
                    cef_value_type_t::VTYPE_BINARY => ValueType::Binary,
                    cef_value_type_t::VTYPE_DICTIONARY => ValueType::Dictionary,
                    cef_value_type_t::VTYPE_LIST => ValueType::List,
                    _ => ValueType::Invalid,
                }
            })
            .unwrap_or(ValueType::Invalid)
    }
    /// Returns the value at the specified key. For simple types the returned value
    /// will copy existing data and modifications to the value will not modify this
    /// object. For complex types (binary, dictionary and list) the returned value
    /// will reference existing data and modifications to the value will modify
    /// this object.
    pub(crate) fn get_value_inner(&self, key: &CefString) -> Value {
        self.0
            .get_value
            .and_then(|get_value| unsafe {
                Value::from_ptr(get_value(self.as_ptr(), key.as_ptr()))
            })
            .unwrap_or_else(Value::new)
    }
    /// Returns the value at the specified key.
    pub fn get(&self, key: &str) -> StoredValue {
        self.get_value_inner(&key.into()).into()
    }
    /// Returns the value at the specified `key` as type bool.
    pub fn get_bool(&self, key: &str) -> bool {
        self.0
            .get_bool
            .map(|get_bool| unsafe { get_bool(self.as_ptr(), CefString::new(key).as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the value at the specified `key` as type int.
    pub fn get_int(&self, key: &str) -> i32 {
        self.0
            .get_int
            .map(|get_int| unsafe { get_int(self.as_ptr(), CefString::new(key).as_ptr()) as i32 })
            .unwrap_or(0)
    }
    /// Returns the value at the specified `key` as type double.
    pub fn get_double(&self, key: &str) -> f64 {
        self.0
            .get_double
            .map(|get_double| unsafe { get_double(self.as_ptr(), CefString::new(key).as_ptr()) })
            .unwrap_or(0.0)
    }
    /// Returns the value at the specified `key` as type string.
    pub fn get_string(&self, key: &str) -> String {
        self.0
            .get_string
            .and_then(|get_string| {
                let s = unsafe { get_string(self.as_ptr(), CefString::new(key).as_ptr()) };
                let result = unsafe { CefString::from_ptr(s).map(String::from) };
                unsafe {
                    cef_string_userfree_utf16_free(s as *mut _);
                }
                result
            })
            .unwrap_or_else(String::new)
    }
    /// Returns the value at the specified key as type binary. The returned value
    /// will reference existing data.
    pub fn try_get_binary(&self, key: &str) -> Option<BinaryValue> {
        self.0.get_binary.and_then(|get_binary| unsafe {
            BinaryValue::from_ptr(get_binary(self.as_ptr(), CefString::new(key).as_ptr()))
        })
    }
    /// Returns the value at the specified key as type dictionary. The returned
    /// value will reference existing data and modifications to the value will
    /// modify this object.
    pub fn try_get_dictionary(&self, key: &str) -> Option<DictionaryValue> {
        self.0.get_dictionary.and_then(|get_dictionary| unsafe {
            DictionaryValue::from_ptr(get_dictionary(self.as_ptr(), CefString::new(key).as_ptr()))
        })
    }
    /// Returns the value at the specified key as type list. The returned value
    /// will reference existing data and modifications to the value will modify
    /// this object.
    pub fn try_get_list(&self, key: &str) -> Option<ListValue> {
        self.0.get_list.and_then(|get_list| unsafe {
            ListValue::from_ptr(get_list(self.as_ptr(), CefString::new(key).as_ptr()))
        })
    }
    /// Sets the value at the specified key. Returns true if the value was set
    /// successfully. If `value` represents simple data then the underlying data
    /// will be copied and modifications to `value` will not modify this object. If
    /// `value` represents complex data (binary, dictionary or list) then the
    /// underlying data will be referenced and modifications to `value` will modify
    /// this object.
    pub(crate) fn insert_inner(&self, key: &str, value: Value) -> bool {
        self.0
            .set_value
            .map(|set_value| unsafe {
                set_value(
                    self.as_ptr(),
                    CefString::new(key).as_ptr(),
                    value.into_raw(),
                ) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key.
    pub(crate) fn insert(&self, key: &str, value: StoredValue) -> bool {
        self.insert_inner(key, value.try_into().unwrap())
    }
    /// Sets the value at the specified key as type null. Returns true if the
    /// value was set successfully.
    pub fn insert_null(&self, key: &str) -> bool {
        self.0
            .set_null
            .map(|set_null| unsafe { set_null(self.as_ptr(), CefString::new(key).as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key as type bool. Returns true if the
    /// value was set successfully.
    pub fn insert_bool(&self, key: &str, value: bool) -> bool {
        self.0
            .set_bool
            .map(|set_bool| unsafe {
                set_bool(
                    self.as_ptr(),
                    CefString::new(key).as_ptr(),
                    if value { 1 } else { 0 },
                ) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key as type int. Returns true if the
    /// value was set successfully.
    pub fn insert_int(&self, key: &str, value: i32) -> bool {
        self.0
            .set_int
            .map(|set_int| unsafe {
                set_int(self.as_ptr(), CefString::new(key).as_ptr(), value) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key as type double. Returns true if the
    /// value was set successfully.
    pub fn insert_double(&self, key: &str, value: f64) -> bool {
        self.0
            .set_double
            .map(|set_double| unsafe {
                set_double(self.as_ptr(), CefString::new(key).as_ptr(), value) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key as type string. Returns true if the
    /// value was set successfully.
    pub fn insert_string(&self, key: &str, value: &str) -> bool {
        self.0
            .set_string
            .map(|set_string| unsafe {
                set_string(
                    self.as_ptr(),
                    CefString::new(key).as_ptr(),
                    CefString::new(value).as_ptr(),
                ) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key as type binary. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn insert_binary(&self, key: &str, value: BinaryValue) -> bool {
        self.0
            .set_binary
            .map(|set_binary| unsafe {
                set_binary(
                    self.as_ptr(),
                    CefString::new(key).as_ptr(),
                    value.into_raw(),
                ) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key as type dict. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn insert_dictionary(&self, key: &str, value: DictionaryValue) -> bool {
        self.0
            .set_dictionary
            .map(|set_dictionary| unsafe {
                set_dictionary(
                    self.as_ptr(),
                    CefString::new(key).as_ptr(),
                    value.into_raw(),
                ) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified key as type list. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn insert_list(&self, key: &str, value: ListValue) -> bool {
        self.0
            .set_list
            .map(|set_list| unsafe {
                set_list(
                    self.as_ptr(),
                    CefString::new(key).as_ptr(),
                    value.into_raw(),
                ) != 0
            })
            .unwrap_or(false)
    }
}

impl Default for DictionaryValue {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<HashMap<String, StoredValue>> for DictionaryValue {
    fn into(self) -> HashMap<String, StoredValue> {
        self.into_iter().collect()
    }
}
impl Into<HashMap<String, StoredValue>> for &'_ DictionaryValue {
    fn into(self) -> HashMap<String, StoredValue> {
        self.into_iter().collect()
    }
}

impl From<&HashMap<String, StoredValue>> for DictionaryValue {
    fn from(map: &HashMap<String, StoredValue>) -> Self {
        let result = Self::new();
        for (key, value) in map {
            if let Ok(value) = Value::try_from(value.clone()) {
                result.insert_inner(key, value);
            }
        }
        result
    }
}

impl PartialEq for DictionaryValue {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.0
            .is_equal
            .map(|is_equal| unsafe { is_equal(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
}

impl crate::cef_helper_traits::DeepClone for DictionaryValue {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn deep_clone(&self) -> Self {
        unsafe { Self::from_ptr_unchecked(self.0.copy.unwrap()(self.as_ptr(), 0)) }
    }
}

impl fmt::Debug for DictionaryValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.into_iter()).finish()
    }
}

ref_counted_ptr! {
    pub struct ListValue(*mut cef_list_value_t);
}

impl ListValue {
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_list_value_create()) }
    }
    /// Returns true if this object is valid. This object may become invalid if
    /// the underlying data is owned by another object (e.g. list or dictionary)
    /// and that other object is then modified or destroyed. Do not call any other
    /// functions if this function returns false.
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the underlying data is owned by another object.
    pub fn is_owned(&self) -> bool {
        self.0
            .is_owned
            .map(|is_owned| unsafe { is_owned(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the underlying data is read-only. Some APIs may expose
    /// read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) != 0 })
            .unwrap_or(true)
    }
    /// Returns true if this object and `that` object have the same underlying
    /// data.
    pub fn is_same(&self, that: &ListValue) -> bool {
        self.0
            .is_same
            .map(|is_same| unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Sets the number of values. If the number of values is expanded all new
    /// value slots will default to type None. Returns true on success.
    pub fn set_len(&self, size: usize) -> bool {
        self.0
            .set_size
            .map(|set_size| unsafe { set_size(self.as_ptr(), size) != 0 })
            .unwrap_or(false)
    }
    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.0
            .get_size
            .map(|get_size| unsafe { get_size(self.as_ptr()) })
            .unwrap_or(0)
    }
    /// Removes all values. Returns true on success.
    pub fn clear(&self) -> bool {
        self.0
            .clear
            .map(|clear| unsafe { clear(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Removes the value at the specified index.
    pub fn remove(&self, index: usize) -> bool {
        self.0
            .remove
            .map(|remove| unsafe { remove(self.as_ptr(), index) != 0 })
            .unwrap_or(false)
    }
    /// Returns the value type at the specified index.
    pub(crate) fn get_type_inner(&self, index: usize) -> ValueType {
        self.0
            .get_type
            .map(|get_type| match unsafe { get_type(self.as_ptr(), index) } {
                cef_value_type_t::VTYPE_NULL => ValueType::Null,
                cef_value_type_t::VTYPE_BOOL => ValueType::Bool,
                cef_value_type_t::VTYPE_INT => ValueType::Int,
                cef_value_type_t::VTYPE_DOUBLE => ValueType::Double,
                cef_value_type_t::VTYPE_STRING => ValueType::String,
                cef_value_type_t::VTYPE_BINARY => ValueType::Binary,
                cef_value_type_t::VTYPE_DICTIONARY => ValueType::Dictionary,
                cef_value_type_t::VTYPE_LIST => ValueType::List,
                _ => ValueType::Invalid,
            })
            .unwrap_or(ValueType::Invalid)
    }
    /// Returns the value at the specified index. For simple types the returned
    /// value will copy existing data and modifications to the value will not
    /// modify this object. For complex types (binary, dictionary and list) the
    /// returned value will reference existing data and modifications to the value
    /// will modify this object.
    pub(crate) fn get_inner(&self, index: usize) -> Option<Value> {
        self.0
            .get_value
            .and_then(|get_value| unsafe { Value::from_ptr(get_value(self.as_ptr(), index)) })
    }
    pub fn get(&self, index: usize) -> Option<StoredValue> {
        self.get_inner(index).map(StoredValue::from)
    }
    /// Returns the value at the specified index as type bool.
    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.0
            .get_bool
            .map(|get_bool| unsafe { get_bool(self.as_ptr(), index) != 0 })
    }
    /// Returns the value at the specified index as type int.
    pub fn get_int(&self, index: usize) -> Option<i32> {
        self.0
            .get_int
            .map(|get_int| unsafe { get_int(self.as_ptr(), index) as i32 })
    }
    /// Returns the value at the specified index as type double.
    pub fn get_double(&self, index: usize) -> Option<f64> {
        self.0
            .get_double
            .map(|get_double| unsafe { get_double(self.as_ptr(), index) })
    }
    /// Returns the value at the specified index as type string.
    pub fn get_string(&self, index: usize) -> Option<String> {
        self.0.get_string.and_then(|get_string| {
            let s = unsafe { get_string(self.as_ptr(), index) };
            let result = unsafe { CefString::from_ptr(s).map(String::from) };
            unsafe {
                cef_string_userfree_utf16_free(s);
            }
            result
        })
    }
    /// Returns the value at the specified index as type binary. The returned value
    /// will reference existing data.
    pub fn get_binary(&self, index: usize) -> Option<BinaryValue> {
        self.0.get_binary.and_then(|get_binary| unsafe {
            BinaryValue::from_ptr(get_binary(self.as_ptr(), index))
        })
    }
    /// Returns the value at the specified index as type dictionary. The returned
    /// value will reference existing data and modifications to the value will
    /// modify this object.
    pub fn get_dictionary(&self, index: usize) -> Option<DictionaryValue> {
        self.0.get_dictionary.and_then(|get_dictionary| unsafe {
            DictionaryValue::from_ptr(get_dictionary(self.as_ptr(), index))
        })
    }
    /// Returns the value at the specified index as type list. The returned value
    /// will reference existing data and modifications to the value will modify
    /// this object.
    pub fn get_list(&self, index: usize) -> Option<ListValue> {
        self.0
            .get_list
            .and_then(|get_list| unsafe { ListValue::from_ptr(get_list(self.as_ptr(), index)) })
    }
    /// Sets the value at the specified index. Returns true if the value was
    /// set successfully. If `value` represents simple data then the underlying
    /// data will be copied and modifications to `value` will not modify this
    /// object. If `value` represents complex data (binary, dictionary or list)
    /// then the underlying data will be referenced and modifications to `value`
    /// will modify this object.
    pub(crate) fn set_value_inner(&self, index: usize, value: Value) -> bool {
        self.0
            .set_value
            .map(|set_value| unsafe { set_value(self.as_ptr(), index, value.into_raw()) != 0 })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type null. Returns true if the
    /// value was set successfully.
    pub fn set_null(&self, index: usize) -> bool {
        self.0
            .set_null
            .map(|set_null| unsafe { set_null(self.as_ptr(), index) != 0 })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type bool. Returns true if the
    /// value was set successfully.
    pub fn set_bool(&self, index: usize, value: bool) -> bool {
        self.0
            .set_bool
            .map(|set_bool| unsafe {
                set_bool(self.as_ptr(), index, if value { 1 } else { 0 }) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type int. Returns true if the
    /// value was set successfully.
    pub fn set_int(&self, index: usize, value: i32) -> bool {
        self.0
            .set_int
            .map(|set_int| unsafe { set_int(self.as_ptr(), index, value) != 0 })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type double. Returns true if the
    /// value was set successfully.
    pub fn set_double(&self, index: usize, value: f64) -> bool {
        self.0
            .set_double
            .map(|set_double| unsafe { set_double(self.as_ptr(), index, value) != 0 })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type string. Returns true if the
    /// value was set successfully.
    pub fn set_string(&self, index: usize, value: &str) -> bool {
        self.0
            .set_string
            .map(|set_string| unsafe {
                set_string(self.as_ptr(), index, CefString::new(value).as_ptr()) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type binary. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn set_binary(&self, index: usize, value: BinaryValue) -> bool {
        self.0
            .set_binary
            .map(|set_binary| unsafe { set_binary(self.as_ptr(), index, value.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type dict. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn set_dictionary(&self, index: usize, value: DictionaryValue) -> bool {
        self.0
            .set_dictionary
            .map(|set_dictionary| unsafe {
                set_dictionary(self.as_ptr(), index, value.into_raw()) != 0
            })
            .unwrap_or(false)
    }
    /// Sets the value at the specified index as type list. Returns true if the
    /// value was set successfully. If `value` is currently owned by another object
    /// then the value will be copied and the `value` reference will not change.
    /// Otherwise, ownership will be transferred to this object and the `value`
    /// reference will be invalidated.
    pub fn set_list(&self, index: usize, value: ListValue) -> bool {
        self.0
            .set_list
            .map(|set_list| unsafe { set_list(self.as_ptr(), index, value.into_raw()) != 0 })
            .unwrap_or(false)
    }
}

impl Default for ListValue {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Vec<StoredValue>> for ListValue {
    fn into(self) -> Vec<StoredValue> {
        self.into_iter().collect()
    }
}

impl fmt::Debug for ListValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.into_iter()).finish()
    }
}

impl PartialEq for ListValue {
    /// Returns true if this object and `that` object have an equivalent
    /// underlying value but are not necessarily the same object.
    fn eq(&self, that: &Self) -> bool {
        self.0
            .is_equal
            .map(|is_equal| unsafe { is_equal(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
}

impl crate::cef_helper_traits::DeepClone for ListValue {
    /// Returns a copy of this object. The underlying data will also be copied.
    fn deep_clone(&self) -> Self {
        unsafe { Self::from_ptr_unchecked((self.0.copy.unwrap())(self.as_ptr())) }
    }
}

/// Structure representing a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point{ x, y }
    }
}

impl From<cef_point_t> for Point {
    fn from(point: cef_point_t) -> Point {
        Point {
            x: point.x,
            y: point.y,
        }
    }
}
impl From<&'_ cef_point_t> for Point {
    fn from(point: &cef_point_t) -> Point {
        Point {
            x: point.x,
            y: point.y,
        }
    }
}
impl From<Point> for cef_point_t {
    fn from(point: Point) -> cef_point_t {
        cef_point_t {
            x: point.x,
            y: point.y,
        }
    }
}

/// Structure representing a rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<cef_rect_t> for Rect {
    fn from(rect: cef_rect_t) -> Rect {
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}
impl From<&'_ cef_rect_t> for Rect {
    fn from(rect: &cef_rect_t) -> Rect {
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}
impl From<Rect> for cef_rect_t {
    fn from(rect: Rect) -> cef_rect_t {
        cef_rect_t {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

/// Structure representing a size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }

    pub(crate) fn as_ptr(&self) -> *const cef_size_t {
        self as *const Size as *const cef_size_t
    }
}

impl From<cef_size_t> for Size {
    fn from(size: cef_size_t) -> Size {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}
impl From<&'_ cef_size_t> for Size {
    fn from(size: &cef_size_t) -> Size {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new()
    }
}

/// Structure representing a range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Range {
    pub from: i32,
    pub to: i32,
}

impl Range {
    pub fn new() -> Self {
        Self {
            from: 0,
            to: 0,
        }
    }

    pub(crate) fn as_ptr(&self) -> *const cef_range_t {
        self as *const Self as *const cef_range_t
    }
}

impl Default for Range {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Range> for cef_range_t {
    fn from(range: Range) -> cef_range_t {
        cef_range_t {
            from: range.from,
            to: range.to,
        }
    }
}

impl From<cef_range_t> for Range {
    fn from(range: cef_range_t) -> Range {
        Range {
            from: range.from,
            to: range.to,
        }
    }
}

/// Structure representing insets.
#[derive(Clone, Debug)]
pub struct Insets {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
}
