use cef_sys::cef_base_ref_counted_t;
use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    os::raw::c_int,
    ptr::NonNull,
    sync::Arc,
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
        $vis:vis struct $Struct:ident$(<$($generic:ident $(: $bound:path)?),+>)?(*mut $cef:ty);
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone)]
        $vis struct $Struct$(<$($generic $(: $bound)?),+>)?(crate::refcounted::RefCountedPtr<$cef>);

        unsafe impl$(<$($generic $(: $bound)?),+>)? Send for $Struct$(<$($generic),+>)? {}
        unsafe impl$(<$($generic $(: $bound)?),+>)? Sync for $Struct$(<$($generic),+>)? {}

        ref_counter!($cef);

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
    object: W,
}

unsafe impl<W: Wrapper> Sync for RefCounted<W> {}
unsafe impl<W: Wrapper> Send for RefCounted<W> {}

impl<W: Wrapper> RefCounted<W> {
    pub(crate) unsafe fn wrapper<'a>(ptr: *mut W::Cef) -> &'a W {
        &(*(ptr as *const W::Cef as *const Self)).object
    }

    pub unsafe fn to_arc(ptr: *mut W::Cef) -> ManuallyDrop<Arc<Self>> {
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
        Arc::into_raw(Arc::new(Self { cefobj, object })) as *mut Self
    }

    pub(crate) extern "C" fn add_ref(ref_counted: *mut cef_base_ref_counted_t) {
        let this = unsafe { Self::to_arc(ref_counted as *mut W::Cef) };
        std::mem::forget(Arc::clone(&*this));
        let _: ManuallyDrop<Arc<Self>> = this;
    }
    pub(crate) extern "C" fn release(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let strong_count = {
            let this: Arc<Self> =
                ManuallyDrop::into_inner(unsafe { Self::to_arc(ref_counted as *mut W::Cef) });
            Arc::strong_count(&this) - 1
        };
        (strong_count == 0) as c_int
    }
    extern "C" fn has_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe { Self::to_arc(ref_counted as *mut W::Cef) };
        (Arc::strong_count(&this) == 1) as c_int
    }
    extern "C" fn has_at_least_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe { Self::to_arc(ref_counted as *mut W::Cef) };
        (Arc::strong_count(&this) >= 1) as c_int
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

// ref_counter!(cef_app_t);
// ref_counter!(cef_browser_process_handler_t);
// ref_counter!(cef_client_t);
// ref_counter!(cef_domvisitor_t);
// ref_counter!(cef_run_file_dialog_callback_t);
// ref_counter!(cef_load_handler_t);
// ref_counter!(cef_render_process_handler_t);
// ref_counter!(cef_request_context_handler_t);
// ref_counter!(cef_resource_bundle_handler_t);
// ref_counter!(cef_resource_request_handler_t);
// ref_counter!(cef_string_visitor_t);
// ref_counter!(cef_urlrequest_client_t);
// ref_counter!(cef_cookie_access_filter_t);
// ref_counter!(cef_response_filter_t);
// ref_counter!(cef_resource_handler_t);
// ref_counter!(cef_download_image_callback_t);
// ref_counter!(cef_pdf_print_callback_t);
// ref_counter!(cef_navigation_entry_visitor_t);
// ref_counter!(cef_task_runner_t);
// ref_counter!(cef_v8handler_t);
// ref_counter!(cef_v8accessor_t);
// ref_counter!(cef_v8interceptor_t);
// ref_counter!(cef_v8array_buffer_release_callback_t);
// ref_counter!(cef_command_line_t);
// ref_counter!(cef_value_t);
// ref_counter!(cef_binary_value_t);
// ref_counter!(cef_dictionary_value_t);
// ref_counter!(cef_list_value_t);
// ref_counter!(cef_image_t);
// ref_counter!(_cef_stream_reader_t);
// ref_counter!(_cef_stream_writer_t);
// ref_counter!(cef_drag_data_t);
// ref_counter!(cef_domdocument_t);
// ref_counter!(cef_domnode_t);
// ref_counter!(cef_process_message_t);
// ref_counter!(cef_request_t);
// ref_counter!(cef_post_data_t);
// ref_counter!(cef_post_data_element_t);
// ref_counter!(cef_frame_t);
// ref_counter!(_cef_x509cert_principal_t);
// ref_counter!(_cef_x509certificate_t);
// ref_counter!(_cef_sslstatus_t);
// ref_counter!(cef_navigation_entry_t);
// ref_counter!(cef_callback_t);
// ref_counter!(_cef_completion_callback_t);
// ref_counter!(_cef_cookie_manager_t);
// ref_counter!(_cef_cookie_visitor_t);
// ref_counter!(_cef_set_cookie_callback_t);
// ref_counter!(_cef_delete_cookies_callback_t);
// ref_counter!(cef_extension_t);
// ref_counter!(_cef_get_extension_resource_callback_t);
// ref_counter!(_cef_extension_handler_t);
// ref_counter!(_cef_resolve_callback_t);
// ref_counter!(cef_request_context_t);
// ref_counter!(cef_browser_t, true);
// ref_counter!(cef_browser_host_t);
// ref_counter!(_cef_print_settings_t);
// ref_counter!(_cef_print_dialog_callback_t);
// ref_counter!(_cef_print_job_callback_t);
// ref_counter!(cef_print_handler_t);
// ref_counter!(_cef_task_t);
// ref_counter!(cef_v8context_t);
// ref_counter!(cef_v8exception_t);
// ref_counter!(cef_v8value_t);
// ref_counter!(cef_v8stack_trace_t);
// ref_counter!(cef_v8stack_frame_t);
// ref_counter!(cef_response_t);
// ref_counter!(cef_resource_skip_callback_t);
// ref_counter!(cef_resource_read_callback_t);
// // ref_counter!(cef_scheme_registrar_t); // doesn't seem to be ref-counted; investigate further as it also has base field
// ref_counter!(_cef_scheme_handler_factory_t);
// ref_counter!(_cef_menu_model_t);
// ref_counter!(_cef_run_context_menu_callback_t);
// ref_counter!(cef_context_menu_handler_t);
// ref_counter!(_cef_context_menu_params_t);
// ref_counter!(_cef_file_dialog_callback_t);
// ref_counter!(cef_dialog_handler_t);
// ref_counter!(cef_display_handler_t);
// ref_counter!(_cef_download_item_t);
// ref_counter!(_cef_before_download_callback_t);
// ref_counter!(_cef_download_item_callback_t);
// ref_counter!(cef_download_handler_t);
// ref_counter!(cef_drag_handler_t);
// ref_counter!(cef_find_handler_t);
// ref_counter!(cef_focus_handler_t);
// ref_counter!(_cef_jsdialog_callback_t);
// ref_counter!(cef_jsdialog_handler_t);
// ref_counter!(cef_keyboard_handler_t);
// ref_counter!(cef_life_span_handler_t);
// ref_counter!(cef_accessibility_handler_t);
// ref_counter!(cef_render_handler_t);
// ref_counter!(cef_auth_callback_t);
// ref_counter!(cef_request_callback_t);
// ref_counter!(_cef_sslinfo_t);
// ref_counter!(_cef_select_client_certificate_callback_t);
// ref_counter!(cef_request_handler_t);
// ref_counter!(cef_urlrequest_t);
// ref_counter!(cef_web_plugin_info_t);
// ref_counter!(cef_web_plugin_info_visitor_t);
// ref_counter!(cef_web_plugin_unstable_callback_t);
// ref_counter!(cef_register_cdm_callback_t);
// ref_counter!(_cef_menu_model_delegate_t);
