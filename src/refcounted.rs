use std::{
    sync::Mutex,
    collections::{HashMap, hash_map::DefaultHasher},
    ptr::hash,
    hash::Hasher,
    os::raw::c_int,
};
use lazy_static::lazy_static;

use cef_sys::cef_base_ref_counted_t;

use crate::ptr_hash::Hashed;

lazy_static! {
    static ref REFCOUNT: Mutex<HashMap<u64, usize>> = Mutex::new(HashMap::new());
}

pub struct RefCounted;

impl RefCounted {
    pub fn wrap<T>(obj: T) -> Box<T> {
        let size = std::mem::size_of::<T>();
        let obj = Box::new(obj);
        let obj_ptr = Box::into_raw(obj);
        let base: *mut cef_base_ref_counted_t = obj_ptr as *mut cef_base_ref_counted_t;
        unsafe {
            (*base).size = size;
            (*base).add_ref = Some(RefCounted::add_ref);
            (*base).release = Some(RefCounted::release::<T>);
            (*base).has_one_ref = Some(RefCounted::has_one_ref);
            (*base).has_at_least_one_ref = Some(RefCounted::has_at_least_one_ref);
        }
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            ref_count.insert(Hashed::from(base).into(), 1);
        }
        unsafe { Box::from_raw(obj_ptr) }
    }

    extern "C" fn add_ref(ref_counted: *mut cef_base_ref_counted_t) {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            if let Some(c) = ref_count.get_mut(&Hashed::from(ref_counted).into()) {
                *c += 1;
            }
        }
    }
    extern "C" fn release<T>(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            let hash = Hashed::from(ref_counted).into();
            if let Some(c) = ref_count.get_mut(&hash) {
                *c -= 1;
                if *c == 0 {
                    ref_count.remove(&hash);
                    unsafe { Box::from_raw(ref_counted as *mut T); }
                    return 1;
                }
            }
        }
        0
    }
    extern "C" fn has_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            let hash = Hashed::from(ref_counted).into();
            if let Some(c) = ref_count.get_mut(&hash) {
                if *c == 1 {
                    return 1;
                }
            }
        }
        0
    }
    extern "C" fn has_at_least_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            let hash = Hashed::from(ref_counted).into();
            if ref_count.contains_key(&hash) {
                return 1;
            }
        }
        0
    }
}