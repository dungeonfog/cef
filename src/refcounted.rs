use cef_sys::cef_base_ref_counted_t;
use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    os::raw::c_int,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};



/// # Safety
/// This trait requires that a pointer to `Self` must also be a valid pointer to a
/// `cef_base_ref_counted_t`. This can be achieved by having the `cef_base_ref_counted_t`
/// field be the first field, and using `#[repr(C)]`. Failure to do will result in undefined
/// behavior.
pub(crate) unsafe trait RefCounter: Sized {
    fn base(&self) -> &cef_base_ref_counted_t;
    fn base_mut(&mut self) -> &mut cef_base_ref_counted_t;
}

pub(crate) trait RefCounterWrapped: RefCounter {
    type Wrapper;
}

macro_rules! ref_counter {
    ($cef:ident) => {
        unsafe impl RefCounter for cef_sys::$cef {
            fn base(&self) -> &cef_base_ref_counted_t {
                &self.base
            }
            fn base_mut(&mut self) -> &mut cef_base_ref_counted_t {
                &mut self.base
            }
        }
    };
    ($cef:ident = $rust:ty) => {
        unsafe impl RefCounter for cef_sys::$cef {
            fn base(&self) -> &cef_base_ref_counted_t {
                &self.base
            }
            fn base_mut(&mut self) -> &mut cef_base_ref_counted_t {
                &mut self.base
            }
        }
        impl RefCounterWrapped for cef_sys::$cef {
            type Wrapper = $rust;
        }
    };
}

/// TODO: PULL OUT OF refcounted.rs
pub trait IsSame {
    fn is_same(this: *mut Self, other: *mut Self) -> bool;
}

macro_rules! is_same {
    ($cef:ident) => {
        impl IsSame for cef_sys::$cef {
            fn is_same(this: *mut Self, other: *mut Self) -> bool {
                unsafe { ((*this).is_same.unwrap())(this, other) != 0 }
            }
        }
    };
}

// TODO: STANDARZIE MEANING OF EQUAL
is_same!(_cef_value_t);
is_same!(_cef_binary_value_t);
is_same!(_cef_dictionary_value_t);
is_same!(_cef_list_value_t);
is_same!(_cef_image_t);
is_same!(_cef_domnode_t);
is_same!(_cef_extension_t);
is_same!(_cef_request_context_t);
is_same!(_cef_browser_t);
is_same!(_cef_task_runner_t);
is_same!(_cef_v8context_t);
is_same!(_cef_v8value_t);

#[repr(transparent)]
pub(crate) struct RefCountedPtr<C: RefCounter> {
    cef: NonNull<C>,
}

impl<C: RefCounter> RefCountedPtr<C> {
    pub(crate) fn wrap(cefobj: C, object: C::Wrapper) -> RefCountedPtr<C>
    where
        C: RefCounterWrapped,
    {
        unsafe { RefCountedPtr::from_ptr_unchecked((*RefCounted::new(cefobj, object)).get_cef()) }
    }

    pub unsafe fn from_ptr_add_ref(ptr: *mut C) -> Option<RefCountedPtr<C>> {
        let mut cef = NonNull::new(ptr)?;
        let add_ref = cef.as_ref().base().add_ref.unwrap();
        (add_ref)(cef.as_mut().base_mut());
        Some(RefCountedPtr { cef })
    }

    pub unsafe fn from_ptr(ptr: *mut C) -> Option<RefCountedPtr<C>> {
        let cef = NonNull::new(ptr)?;
        Some(RefCountedPtr { cef })
    }

    pub unsafe fn from_ptr_unchecked(ptr: *mut C) -> RefCountedPtr<C> {
        let cef = NonNull::new_unchecked(ptr);
        RefCountedPtr { cef }
    }

    pub fn as_ptr(&self) -> *mut C {
        self.cef.as_ptr()
    }

    pub fn into_raw(self) -> *mut C {
        let ptr = self.cef.as_ptr();
        std::mem::forget(self);
        ptr
    }
}

macro_rules! ref_counted_ptr {
    (
        $(#[$meta:meta])*
        $vis:vis struct $Struct:ident(*mut $cef:ident);
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        $vis struct $Struct(crate::refcounted::RefCountedPtr<cef_sys::$cef>);

        impl $Struct {
            pub unsafe fn from_ptr_add_ref(ptr: *mut $cef) -> Option<$Struct> {
                crate::refcounted::RefCountedPtr::from_ptr_add_ref(ptr).map(Self)
            }

            pub unsafe fn from_ptr(ptr: *mut $cef) -> Option<$Struct> {
                crate::refcounted::RefCountedPtr::from_ptr(ptr).map(Self)
            }

            pub unsafe fn from_ptr_unchecked(ptr: *mut $cef) -> $Struct {
                Self(crate::refcounted::RefCountedPtr::from_ptr_unchecked(ptr))
            }

            pub fn as_ptr(&self) -> *mut $cef {
                self.0.as_ptr()
            }

            pub fn into_raw(self) -> *mut $cef {
                self.0.into_raw()
            }
        }
    };
}

impl<C: RefCounter> Deref for RefCountedPtr<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        unsafe { self.cef.as_ref() }
    }
}

impl<C: RefCounter> DerefMut for RefCountedPtr<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.cef.as_mut() }
    }
}

impl<C: RefCounter> Drop for RefCountedPtr<C> {
    fn drop(&mut self) {
        unsafe {
            let release = self.cef.as_ref().base().release.unwrap();
            let base = self.cef.as_mut().base_mut();
            (release)(base);
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
pub(crate) struct RefCounted<C: RefCounterWrapped> {
    cefobj: C,
    refcount: AtomicUsize,
    object: C::Wrapper,
}

unsafe impl<C: RefCounterWrapped> Sync for RefCounted<C> {}
unsafe impl<C: RefCounterWrapped> Send for RefCounted<C> {}

impl<C: RefCounterWrapped> Deref for RefCounted<C> {
    type Target = C::Wrapper;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<C: RefCounterWrapped> DerefMut for RefCounted<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<C: RefCounterWrapped> RefCounted<C> {
    pub(crate) unsafe fn make_temp(ptr: *mut C) -> ManuallyDrop<Box<Self>> {
        ManuallyDrop::new(Box::from_raw(ptr as *mut Self))
    }

    pub(crate) fn new(mut cefobj: C, object: C::Wrapper) -> *mut Self {
        *cefobj.base_mut() = cef_base_ref_counted_t {
            size: std::mem::size_of::<C>(),
            add_ref: Some(Self::add_ref),
            release: Some(Self::release),
            has_one_ref: Some(Self::has_one_ref),
            has_at_least_one_ref: Some(Self::has_at_least_one_ref),
        };

        Box::into_raw(Box::new(Self {
            cefobj,
            refcount: AtomicUsize::new(1),
            object,
        }))
    }

    pub(crate) fn get_cef(&mut self) -> *mut C {
        &mut self.cefobj as *mut C
    }

    pub(crate) extern "C" fn add_ref(ref_counted: *mut cef_base_ref_counted_t) {
        let this = unsafe { Self::make_temp(ref_counted as *mut C) };
        this.refcount.fetch_add(1, Ordering::AcqRel);
    }
    pub(crate) extern "C" fn release(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe { Self::make_temp(ref_counted as *mut C) };
        if this.refcount.fetch_sub(1, Ordering::AcqRel) < 1 {
            ManuallyDrop::into_inner(this);
            0
        } else {
            1
        }
    }
    extern "C" fn has_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe { Self::make_temp(ref_counted as *mut C) };
        let counter = this.refcount.load(Ordering::Acquire);
        if counter == 1 {
            1
        } else {
            0
        }
    }
    extern "C" fn has_at_least_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let this = unsafe { Self::make_temp(ref_counted as *mut C) };
        let counter = this.refcount.load(Ordering::Acquire);
        if counter >= 1 {
            1
        } else {
            0
        }
    }
}

ref_counter!(cef_app_t = crate::app::AppWrapper);
ref_counter!(
    cef_browser_process_handler_t = crate::browser_process_handler::BrowserProcessHandlerWrapper
);
ref_counter!(cef_client_t = crate::client::ClientWrapper);
ref_counter!(cef_domvisitor_t = Box<dyn crate::dom::DOMVisitor>);
ref_counter!(cef_run_file_dialog_callback_t = Option<Box<dyn FnOnce(usize, Option<Vec<String>>)>>);
ref_counter!(cef_load_handler_t = Box<dyn crate::load_handler::LoadHandler>);
ref_counter!(
    cef_render_process_handler_t = crate::render_process_handler::RenderProcessHandlerWrapper
);
ref_counter!(cef_request_context_handler_t = crate::request_context::RequestContextHandlerWrapper);
ref_counter!(cef_resource_bundle_handler_t = Box<dyn crate::resource_bundle_handler::ResourceBundleHandler>);
ref_counter!(
    cef_resource_request_handler_t = crate::resource_request_handler::ResourceRequestHandlerWrapper
);
ref_counter!(cef_string_visitor_t = Box<dyn crate::string::StringVisitor>);
ref_counter!(cef_urlrequest_client_t = Box<dyn crate::url_request::URLRequestClient>);
ref_counter!(cef_cookie_access_filter_t = Box<dyn crate::url_request::CookieAccessFilter>);
ref_counter!(cef_response_filter_t = Box<dyn crate::url_request::ResponseFilter>);
ref_counter!(cef_resource_handler_t = Box<dyn crate::url_request::ResourceHandler>);
ref_counter!(cef_command_line_t);
ref_counter!(cef_value_t);
ref_counter!(cef_binary_value_t);
ref_counter!(cef_dictionary_value_t);
ref_counter!(cef_list_value_t);
ref_counter!(cef_image_t);
ref_counter!(_cef_stream_reader_t);
ref_counter!(_cef_stream_writer_t);
ref_counter!(cef_drag_data_t);
ref_counter!(cef_domdocument_t);
ref_counter!(cef_domnode_t);
ref_counter!(cef_process_message_t);
ref_counter!(cef_request_t);
ref_counter!(cef_post_data_t);
ref_counter!(cef_post_data_element_t);
ref_counter!(cef_frame_t);
ref_counter!(_cef_x509cert_principal_t);
ref_counter!(_cef_x509certificate_t);
ref_counter!(_cef_sslstatus_t);
ref_counter!(cef_navigation_entry_t);
ref_counter!(cef_callback_t);
ref_counter!(_cef_completion_callback_t);
ref_counter!(_cef_cookie_manager_t);
ref_counter!(_cef_cookie_visitor_t);
ref_counter!(_cef_set_cookie_callback_t);
ref_counter!(_cef_delete_cookies_callback_t);
ref_counter!(cef_extension_t);
ref_counter!(_cef_get_extension_resource_callback_t);
ref_counter!(_cef_extension_handler_t);
ref_counter!(_cef_resolve_callback_t);
ref_counter!(cef_request_context_t);
ref_counter!(cef_browser_t);
ref_counter!(_cef_navigation_entry_visitor_t);
ref_counter!(_cef_pdf_print_callback_t);
ref_counter!(_cef_download_image_callback_t);
ref_counter!(cef_browser_host_t);
ref_counter!(_cef_print_settings_t);
ref_counter!(_cef_print_dialog_callback_t);
ref_counter!(_cef_print_job_callback_t);
ref_counter!(cef_print_handler_t);
ref_counter!(_cef_task_t);
ref_counter!(_cef_task_runner_t);
ref_counter!(cef_v8context_t);
ref_counter!(cef_v8handler_t);
ref_counter!(cef_v8accessor_t);
ref_counter!(cef_v8interceptor_t);
ref_counter!(cef_v8exception_t);
ref_counter!(cef_v8array_buffer_release_callback_t);
ref_counter!(cef_v8value_t);
ref_counter!(cef_v8stack_trace_t);
ref_counter!(cef_v8stack_frame_t);
ref_counter!(cef_response_t);
ref_counter!(cef_resource_skip_callback_t);
ref_counter!(cef_resource_read_callback_t);
// ref_counter!(cef_scheme_registrar_t); // doesn't seem to be ref-counted; investigate further as it also has base field
ref_counter!(_cef_scheme_handler_factory_t);
ref_counter!(_cef_menu_model_t);
ref_counter!(_cef_run_context_menu_callback_t);
ref_counter!(cef_context_menu_handler_t);
ref_counter!(_cef_context_menu_params_t);
ref_counter!(_cef_file_dialog_callback_t);
ref_counter!(cef_dialog_handler_t);
ref_counter!(cef_display_handler_t);
ref_counter!(_cef_download_item_t);
ref_counter!(_cef_before_download_callback_t);
ref_counter!(_cef_download_item_callback_t);
ref_counter!(cef_download_handler_t);
ref_counter!(cef_drag_handler_t);
ref_counter!(cef_find_handler_t);
ref_counter!(cef_focus_handler_t);
ref_counter!(_cef_jsdialog_callback_t);
ref_counter!(cef_jsdialog_handler_t);
ref_counter!(cef_keyboard_handler_t);
ref_counter!(cef_life_span_handler_t);
ref_counter!(cef_accessibility_handler_t);
ref_counter!(cef_render_handler_t);
ref_counter!(cef_auth_callback_t);
ref_counter!(cef_request_callback_t);
ref_counter!(_cef_sslinfo_t);
ref_counter!(_cef_select_client_certificate_callback_t);
ref_counter!(cef_request_handler_t);
ref_counter!(cef_urlrequest_t);
ref_counter!(cef_web_plugin_info_t);
ref_counter!(cef_web_plugin_info_visitor_t);
ref_counter!(cef_web_plugin_unstable_callback_t);
ref_counter!(cef_register_cdm_callback_t);
