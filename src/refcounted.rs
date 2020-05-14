use cef_sys::cef_base_ref_counted_t;
use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    os::raw::c_int,
    ptr::NonNull,
    sync::{Arc, atomic::{self, AtomicUsize, Ordering}},
};
use chashmap::CHashMap;
use lazy_static::lazy_static;

/// # Safety
/// This trait requires that a pointer to `Self` must also be a valid pointer to a
/// `cef_base_ref_counted_t`. This can be achieved by having the `cef_base_ref_counted_t`
/// field be the first field, and using `#[repr(C)]`. Failure to do will result in undefined
/// behavior.
pub(crate) unsafe trait RefCounter: Sized {
    const POISONABLE: bool;
    fn base(&self) -> &cef_base_ref_counted_t;
    fn base_mut(&mut self) -> &mut cef_base_ref_counted_t;
}

pub(crate) trait Wrapper: Sized + Send + Sync {
    type Cef: RefCounter;
    fn wrap(self) -> RefCountedPtr<Self::Cef>;
}

macro_rules! ref_counter {
    ($cef:ty) => {
        ref_counter!($cef, false);
    };
    ($cef:ty, $poisonable:expr) => {
        unsafe impl crate::refcounted::RefCounter for $cef {
            const POISONABLE: bool = $poisonable;
            fn base(&self) -> &cef_sys::cef_base_ref_counted_t {
                &self.base
            }
            fn base_mut(&mut self) -> &mut cef_sys::cef_base_ref_counted_t {
                &mut self.base
            }
        }
    };
}

#[repr(transparent)]
pub(crate) struct RefCountedPtr<C: RefCounter> {
    cef: NonNull<C>,
}

unsafe impl<C: RefCounter> Send for RefCountedPtr<C> {}
unsafe impl<C: RefCounter> Sync for RefCountedPtr<C> {}

impl<C: RefCounter> RefCountedPtr<C> {
    pub(crate) fn wrap<W: Wrapper<Cef = C>>(cefobj: C, object: W) -> RefCountedPtr<C> {
        unsafe { RefCountedPtr::from_ptr_unchecked(RefCounted::new(cefobj, object) as *mut C) }
    }

    pub(crate) unsafe fn from_ptr_add_ref(ptr: *mut C) -> Option<RefCountedPtr<C>> {
        let mut cef = NonNull::new(ptr)?;
        let add_ref = cef.as_ref().base().add_ref.unwrap();
        (add_ref)(cef.as_mut().base_mut());
        Some(RefCountedPtr { cef })
    }

    pub(crate) unsafe fn from_ptr(ptr: *mut C) -> Option<RefCountedPtr<C>> {
        let cef = NonNull::new(ptr)?;
        Some(RefCountedPtr { cef })
    }

    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut C) -> RefCountedPtr<C> {
        debug_assert!(ptr != std::ptr::null_mut());
        let cef = NonNull::new_unchecked(ptr);
        RefCountedPtr { cef }
    }

    pub(crate) fn as_ptr(&self) -> *mut C {
        self.cef.as_ptr()
    }

    pub(crate) fn into_raw(self) -> *mut C {
        let ptr = self.cef.as_ptr();
        std::mem::forget(self);
        ptr
    }

    pub(crate) unsafe fn poison(self) {
        if C::POISONABLE {
            POISON_TABLE.insert(self.cef.as_ptr() as usize, ());
        } else {
            panic!("not poisonable");
        }
    }

    fn check_poisoned(&self) -> bool {
        if C::POISONABLE {
            POISON_TABLE.contains_key(&(self.cef.as_ptr() as usize))
        } else {
            false
        }
    }
}

lazy_static!{
    static ref POISON_TABLE: CHashMap<usize, ()> = CHashMap::new();
}

macro_rules! ref_counted_ptr {
    (
        $(#[$meta:meta])*
        $vis:vis struct $Struct:ident$(<$($generic:ident $(: $bound:path)?),+>)?(*mut $cef:ty $(, $poisonable:expr)?);
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone)]
        $vis struct $Struct$(<$($generic $(: $bound)?),+>)?(crate::refcounted::RefCountedPtr<$cef>);

        unsafe impl$(<$($generic $(: $bound)?),+>)? Send for $Struct$(<$($generic),+>)? {}
        unsafe impl$(<$($generic $(: $bound)?),+>)? Sync for $Struct$(<$($generic),+>)? {}

        ref_counter!($cef $(, $poisonable)?);

        impl$(<$($generic $(: $bound)?),+>)? $Struct$(<$($generic),+>)? {
            pub(crate) unsafe fn from_ptr_add_ref(ptr: *mut $cef) -> Option<Self> {
                crate::refcounted::RefCountedPtr::from_ptr_add_ref(ptr).map(Self)
            }

            pub(crate) unsafe fn from_ptr(ptr: *mut $cef) -> Option<Self> {
                crate::refcounted::RefCountedPtr::from_ptr(ptr).map(Self)
            }

            pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut $cef) -> Self {
                Self(crate::refcounted::RefCountedPtr::from_ptr_unchecked(ptr))
            }

            pub(crate) unsafe fn from_ptr_ptr<'a>(ptr: *mut *mut $cef) -> &'a mut Self {
                &mut *(ptr as *mut Self)
            }

            pub(crate) fn as_ptr(&self) -> *mut $cef {
                self.0.as_ptr()
            }

            pub(crate) fn into_raw(self) -> *mut $cef {
                self.0.into_raw()
            }

            pub(crate) unsafe fn poison(self) {
                self.0.poison()
            }
        }

        owned_casts!(impl for $Struct = *mut $cef);
    };
}

impl<C: RefCounter> Deref for RefCountedPtr<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        if self.check_poisoned() {
            panic!("Attempted to use poisoned struct");
        }
        unsafe { self.cef.as_ref() }
    }
}

impl<C: RefCounter> DerefMut for RefCountedPtr<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.check_poisoned() {
            panic!("Attempted to use poisoned struct");
        }
        unsafe { self.cef.as_mut() }
    }
}

impl<C: RefCounter> Drop for RefCountedPtr<C> {
    fn drop(&mut self) {
        unsafe {
            if !self.check_poisoned() {
                let release = self.cef.as_ref().base().release.unwrap();
                let base = self.cef.as_mut().base_mut();
                (release)(base);
            } else {
                println!("drop poisoned")
            }
        }
    }
}

impl<C: RefCounter> Clone for RefCountedPtr<C> {
    fn clone(&self) -> RefCountedPtr<C> {
        unsafe {
            let mut new = RefCountedPtr { cef: self.cef };
            let add_ref = new.cef.as_ref().base().add_ref.unwrap();
            (add_ref)(new.cef.as_mut().base_mut());
            new
        }
    }
}

// The code for RefCounted<C,R> assumes that it can cast *mut cef_base_ref_counted_t to *mut C to *mut RefCounted<C,R>
// this is true as long as everything is #[repr(C)] and the corresponding structs are the first in the list.
// It might sound like a hack, but I think that CEF assumes that you do it like this. It's a C API after all.

#[repr(C)]
pub(crate) struct RefCounted<W: Wrapper> {
    cefobj: W::Cef,
    /// The ref-counting implementation here is blatantly stolen from `std::sync::Arc`, since it's
    /// a well-tested and robust design. There's lots of good documentation in the standard
    /// library source on the rationale for the various atomic operations, so reference that code for
    /// explanations. See https://github.com/dungeonfog/cef/issues/1 for why we aren't just using `Arc`
    /// directly.
    ref_count: AtomicUsize,
    object: W,
}

unsafe impl<W: Wrapper> Sync for RefCounted<W> {}
unsafe impl<W: Wrapper> Send for RefCounted<W> {}

impl<W: Wrapper> RefCounted<W> {
    pub(crate) unsafe fn wrapper<'a>(ptr: *mut W::Cef) -> &'a W {
        &(*(ptr as *const W::Cef as *const Self)).object
    }

    unsafe fn to_arc(ptr: *mut W::Cef) -> ManuallyDrop<Arc<Self>> {
        ManuallyDrop::new(Arc::from_raw(ptr as *mut Self))
    }

    pub(crate) fn new(mut cefobj: W::Cef, object: W) -> *mut Self {
        *cefobj.base_mut() = cef_base_ref_counted_t {
            size: std::mem::size_of::<W::Cef>(),
            add_ref: Some(Self::add_ref),
            release: Some(Self::release),
            has_one_ref: Some(Self::has_one_ref),
            has_at_least_one_ref: Some(Self::has_at_least_one_ref),
        };

        // TODO: SHOULD WE GIT RID OF THE POINTER CAST? THIS IS BEING SHARED ACROSS THREADS AFTER
        // ALL
        Box::into_raw(Box::new(Self {
            cefobj,
            ref_count: AtomicUsize::new(1),
            object,
        }))
    }

    pub(crate) extern "C" fn add_ref(ref_counted: *mut cef_base_ref_counted_t) {
        let this = unsafe{ &*(ref_counted as *const Self) };
        let old_size = this.ref_count.fetch_add(1, Ordering::Relaxed);
        if old_size == usize::max_value() {
            panic!("ref_count too big!");
        }
    }
    pub(crate) extern "C" fn release(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe{ &*(ref_counted as *const Self) };
        let strong_count = this.ref_count.fetch_sub(1, Ordering::Release);
        atomic::fence(Ordering::Acquire);

        if strong_count == 1 {
            unsafe{ Box::from_raw(ref_counted as *mut Self); }
            1
        } else {
            0
        }
    }
    extern "C" fn has_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe{ &*(ref_counted as *const Self) };
        (this.ref_count.load(Ordering::SeqCst) == 1) as c_int
    }
    extern "C" fn has_at_least_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe{ &*(ref_counted as *const Self) };
        (this.ref_count.load(Ordering::SeqCst) >= 1) as c_int
    }
}

unsafe impl RefCounter for cef_base_ref_counted_t {
    const POISONABLE: bool = false;
    fn base(&self) -> &cef_base_ref_counted_t {
        self
    }
    fn base_mut(&mut self) -> &mut cef_base_ref_counted_t {
        self
    }
}
