use cef_sys::{
    cef_string_list_alloc, cef_string_list_append, cef_string_list_free, cef_string_list_size,
    cef_string_list_t, cef_string_list_value, cef_string_t, cef_string_utf8_to_utf16,
    cef_string_visitor_t, cef_string_userfree_t, cef_string_userfree_utf16_free,
    cef_string_userfree_utf16_alloc,
};
use std::ptr::null_mut;

use std::{iter::FromIterator, mem, ops::Range};
use parking_lot::Mutex;

use crate::refcounted::{RefCountedPtr, Wrapper};

#[repr(transparent)]
pub(crate) struct CefString(cef_string_t);

impl CefString {
    pub fn null() -> Self {
        Self(cef_string_t {
            str: null_mut(),
            length: 0,
            dtor: None,
        })
    }
    pub fn new(source: &str) -> Self {
        let mut instance = unsafe { std::mem::zeroed() };
        let len = source.len();
        unsafe {
            cef_string_utf8_to_utf16(
                source.as_ptr() as *const std::os::raw::c_char,
                len,
                &mut instance,
            );
        }
        CefString(instance)
    }
    pub fn set_string(&mut self, str: &str) {
        unsafe {
            cef_string_utf8_to_utf16(
                str.as_ptr() as *const std::os::raw::c_char,
                str.len(),
                &mut self.0,
            );
        }
    }
    pub fn into_raw(self) -> cef_string_t {
        let result = cef_string_t { ..self.0 };
        mem::forget(self);
        result
    }
    pub fn as_ptr(&self) -> *const cef_string_t {
        &self.0
    }
    pub fn as_ptr_mut(&mut self) -> *mut cef_string_t {
        &mut self.0
    }

    pub unsafe fn from_ptr<'a>(ptr: *const cef_string_t) -> Option<&'a CefString> {
        assert_eq!(
            std::mem::size_of::<cef_string_t>(),
            std::mem::size_of::<CefString>()
        );
        (ptr as *const CefString).as_ref()
    }
    pub unsafe fn from_ptr_unchecked<'a>(ptr: *const cef_string_t) -> &'a CefString {
        assert_eq!(
            std::mem::size_of::<cef_string_t>(),
            std::mem::size_of::<CefString>()
        );
        &*(ptr as *const CefString)
    }
    pub unsafe fn from_mut_ptr<'a>(ptr: *mut cef_string_t) -> Option<&'a mut CefString> {
        assert_eq!(
            std::mem::size_of::<cef_string_t>(),
            std::mem::size_of::<CefString>()
        );
        (ptr as *mut CefString).as_mut()
    }
    pub unsafe fn from_mut_ptr_unchecked<'a>(ptr: *mut cef_string_t) -> &'a mut CefString {
        assert_eq!(
            std::mem::size_of::<cef_string_t>(),
            std::mem::size_of::<CefString>()
        );
        &mut *(ptr as *mut CefString)
    }

    pub unsafe fn from_raw(raw: cef_string_t) -> CefString {
        CefString(raw)
    }

    pub unsafe fn from_userfree(raw: cef_string_userfree_t) -> Option<CefString> {
        if raw == null_mut() {
            None
        } else {
            Some(CefString(unwrap_userfree(raw)))
        }
    }

    pub unsafe fn from_userfree_unchecked(raw: cef_string_userfree_t) -> CefString {
        CefString(unwrap_userfree(raw))
    }
}

/// De-allocate the structure the userfree points to without freeing the underlying buffers.
unsafe fn unwrap_userfree(raw: cef_string_userfree_t) -> cef_string_t {
    unsafe extern "C" fn null_dtor(str: *mut u16) {}
    let cef_string = cef_string_t{..*raw};
    (*raw).dtor = Some(null_dtor);
    cef_string_userfree_utf16_free(raw);
    cef_string
}

impl Default for CefString {
    fn default() -> Self {
        CefString(unsafe { std::mem::zeroed() })
    }
}

impl Drop for CefString {
    fn drop(&mut self) {
        if let Some(dtor) = self.0.dtor {
            unsafe {
                dtor(self.0.str);
            }
        }
    }
}

impl From<cef_string_t> for CefString {
    fn from(source: cef_string_t) -> Self {
        CefString(source)
    }
}

impl From<CefString> for cef_string_userfree_t {
    fn from(string: CefString) -> cef_string_userfree_t {
        unsafe {
            let userfree = cef_string_userfree_utf16_alloc();
            *userfree = string.into_raw();
            userfree
        }
    }
}

impl<'a> From<&'a str> for CefString {
    fn from(s: &'a str) -> CefString {
        CefString::new(s)
    }
}

impl From<CefString> for String {
    fn from(cef: CefString) -> String {
        String::from(&cef)
    }
}

impl<'a> From<&'a mut CefString> for String {
    fn from(cef: &'a mut CefString) -> String {
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(cef.0.str, cef.0.length) })
    }
}

impl<'a> From<&'a CefString> for String {
    fn from(cef: &'a CefString) -> String {
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(cef.0.str, cef.0.length) })
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
        unsafe {
            cef_string_list_free(self.0);
        }
    }
}

impl From<CefStringList> for cef_string_list_t {
    fn from(list: CefStringList) -> cef_string_list_t {
        list.into_raw()
    }
}

impl CefStringList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn as_mut_ptr(&mut self) -> cef_string_list_t {
        self.0
    }
    pub fn len(&self) -> usize {
        unsafe { cef_string_list_size(self.0) }
    }
    pub fn get(&self, index: usize) -> Option<CefString> {
        let mut string = CefString::default();
        let result = unsafe { cef_string_list_value(self.0, index, string.as_ptr_mut()) };
        if result == 0 {
            None
        } else {
            Some(string)
        }
    }
    pub fn push(&mut self, s: &CefString) {
        unsafe {
            cef_string_list_append(self.0, s.as_ptr());
        }
    }
    pub unsafe fn from_raw(raw: cef_string_list_t) -> mem::ManuallyDrop<CefStringList> {
        mem::ManuallyDrop::new(CefStringList(raw))
    }
    pub fn into_raw(self) -> cef_string_list_t {
        let list = self.0;
        mem::forget(self);
        list
    }
}

impl<'a> IntoIterator for &'a CefStringList {
    type Item = CefString;
    type IntoIter = CefStringListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CefStringListIter {
            list: self,
            range: 0..self.len(),
        }
    }
}

impl IntoIterator for CefStringList {
    type Item = CefString;
    type IntoIter = CefStringListIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        CefStringListIntoIter {
            range: 0..self.len(),
            list: self,
        }
    }
}

pub(crate) struct CefStringListIter<'a> {
    list: &'a CefStringList,
    range: Range<usize>,
}

pub(crate) struct CefStringListIntoIter {
    list: CefStringList,
    range: Range<usize>,
}

impl<'a> Iterator for CefStringListIter<'a> {
    type Item = CefString;

    fn next(&mut self) -> Option<CefString> {
        self.range.next().and_then(|i| self.list.get(i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = self.range.len();
        (s, Some(s))
    }
}
impl<'a> ExactSizeIterator for CefStringListIter<'a> {}

impl Iterator for CefStringListIntoIter {
    type Item = CefString;

    fn next(&mut self) -> Option<CefString> {
        self.range.next().and_then(|i| self.list.get(i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = self.range.len();
        (s, Some(s))
    }
}
impl<'a> ExactSizeIterator for CefStringListIntoIter {}

impl<'a> FromIterator<&'a str> for CefStringList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<'a> Extend<&'a str> for CefStringList {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a str>,
    {
        for s in iter {
            self.push(&s.into());
        }
    }
}

impl<'a> Extend<&'a CefString> for CefStringList {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = &'a CefString>,
    {
        for s in iter {
            self.push(s);
        }
    }
}

impl From<CefStringList> for Vec<String> {
    fn from(list: CefStringList) -> Self {
        Vec::from_iter(list.into_iter().map(String::from))
    }
}

impl From<&'_ CefStringList> for Vec<String> {
    fn from(list: &CefStringList) -> Self {
        Vec::from_iter(list.into_iter().map(String::from))
    }
}

ref_counted_ptr!{
    pub struct StringVisitor(*mut cef_string_visitor_t);
}

impl StringVisitor {
    pub fn new<C: StringVisitorCallback>(callback: C) -> StringVisitor {
        unsafe{ StringVisitor::from_ptr_unchecked(StringVisitorWrapper::new(Box::new(callback)).wrap().into_raw()) }
    }
}

pub trait StringVisitorCallback = 'static + Send + FnMut(&str);

pub(crate) struct StringVisitorWrapper {
    delegate: Mutex<Box<dyn StringVisitorCallback>>,
}

impl Wrapper for StringVisitorWrapper {
    type Cef = cef_string_visitor_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_string_visitor_t {
                base: unsafe { std::mem::zeroed() },
                visit: Some(Self::visit),
            },
            self,
        )
    }
}

impl StringVisitorWrapper {
    pub(crate) fn new(delegate: Box<dyn StringVisitorCallback>) -> StringVisitorWrapper {
        StringVisitorWrapper { delegate: Mutex::new(delegate) }
    }
}

cef_callback_impl! {
    impl for StringVisitorWrapper: cef_string_visitor_t {
        fn visit(
            &self,
            string: &CefString: *const cef_string_t
        ) {
            (&mut *self.delegate.lock())(&String::from(string))
        }
    }
}
