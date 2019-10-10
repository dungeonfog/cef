use cef_sys::cef_base_ref_counted_t;
use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    os::raw::c_int,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::ptr_hash::Hashed;

pub(crate) trait RefCounter {
    type Wrapper;
    fn set_base(&mut self, base: cef_base_ref_counted_t);
}

macro_rules! ref_counter {
    ($cef:ident = $rust:ty) => {
        impl RefCounter for cef_sys::$cef {
            type Wrapper = $rust;
            fn set_base(&mut self, base: cef_base_ref_counted_t) {
                self.base = base;
            }
        }
    };
}

// The code for RefCounted<C,R> assumes that it can cast *mut cef_base_ref_counted_t to *mut C to *mut RefCounted<C,R>
// this is true as long as everything is #[repr(C)] and the corresponding structs are the first in the list.
// It might sound like a hack, but I think that CEF assumes that you do it like this. It's a C API after all.
#[repr(C)]
pub(crate) struct RefCounted<C: RefCounter + Sized> {
    cefobj: C,
    refcount: AtomicUsize,
    object: C::Wrapper,
}

unsafe impl<C: RefCounter + Sized> Sync for RefCounted<C> {}
unsafe impl<C: RefCounter + Sized> Send for RefCounted<C> {}

impl<C: RefCounter + Sized> Deref for RefCounted<C> {
    type Target = C::Wrapper;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<C: RefCounter + Sized> DerefMut for RefCounted<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<C: RefCounter + Sized> RefCounted<C> {
    pub(crate) unsafe fn make_temp(ptr: *mut C) -> ManuallyDrop<Box<Self>> {
        ManuallyDrop::new(unsafe { Box::from_raw(ptr as *mut Self) })
    }

    pub(crate) fn new(mut cefobj: C, object: C::Wrapper) -> *mut Self {
        cefobj.set_base(cef_base_ref_counted_t {
            size: std::mem::size_of::<C>(),
            add_ref: Some(Self::add_ref),
            release: Some(Self::release),
            has_one_ref: Some(Self::has_one_ref),
            has_at_least_one_ref: Some(Self::has_at_least_one_ref),
        });

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
        let mut this = unsafe { Self::make_temp(ref_counted as *mut C) };
        this.refcount.fetch_add(1, Ordering::AcqRel);
    }
    pub(crate) extern "C" fn release(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let mut this = unsafe { Self::make_temp(ref_counted as *mut C) };
        if this.refcount.fetch_sub(1, Ordering::AcqRel) < 1 {
            ManuallyDrop::into_inner(this);
            0
        } else {
            1
        }
    }
    extern "C" fn has_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let mut this = unsafe { Self::make_temp(ref_counted as *mut C) };
        let counter = this.refcount.load(Ordering::Acquire);
        if counter == 1 {
            1
        } else {
            0
        }
    }
    extern "C" fn has_at_least_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        let mut this = unsafe { Self::make_temp(ref_counted as *mut C) };
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
ref_counter!(cef_command_line_t = ());
ref_counter!(cef_value_t = ());
ref_counter!(cef_binary_value_t = ());
ref_counter!(cef_dictionary_value_t = ());
ref_counter!(cef_list_value_t = ());
ref_counter!(cef_image_t = ());
ref_counter!(_cef_stream_reader_t = ());
ref_counter!(_cef_stream_writer_t = ());
ref_counter!(cef_drag_data_t = ());
ref_counter!(cef_domdocument_t = ());
ref_counter!(cef_domnode_t = ());
ref_counter!(cef_process_message_t = ());
ref_counter!(cef_request_t = ());
ref_counter!(cef_post_data_t = ());
ref_counter!(cef_post_data_element_t = ());
ref_counter!(cef_frame_t = ());
ref_counter!(_cef_x509cert_principal_t = ());
ref_counter!(_cef_x509certificate_t = ());
ref_counter!(_cef_sslstatus_t = ());
ref_counter!(cef_navigation_entry_t = ());
ref_counter!(cef_callback_t = ());
ref_counter!(_cef_completion_callback_t = ());
ref_counter!(_cef_cookie_manager_t = ());
ref_counter!(_cef_cookie_visitor_t = ());
ref_counter!(_cef_set_cookie_callback_t = ());
ref_counter!(_cef_delete_cookies_callback_t = ());
ref_counter!(_cef_extension_t = ());
ref_counter!(_cef_get_extension_resource_callback_t = ());
ref_counter!(_cef_extension_handler_t = ());
ref_counter!(_cef_resolve_callback_t = ());
ref_counter!(cef_request_context_t = ());
ref_counter!(cef_browser_t = ());
ref_counter!(_cef_navigation_entry_visitor_t = ());
ref_counter!(_cef_pdf_print_callback_t = ());
ref_counter!(_cef_download_image_callback_t = ());
ref_counter!(cef_browser_host_t = ());
ref_counter!(_cef_print_settings_t = ());
ref_counter!(_cef_print_dialog_callback_t = ());
ref_counter!(_cef_print_job_callback_t = ());
ref_counter!(cef_print_handler_t = ());
ref_counter!(_cef_task_t = ());
ref_counter!(_cef_task_runner_t = ());
ref_counter!(cef_v8context_t = ());
ref_counter!(cef_v8handler_t = ());
ref_counter!(cef_v8accessor_t = ());
ref_counter!(cef_v8interceptor_t = ());
ref_counter!(cef_v8exception_t = ());
ref_counter!(cef_v8array_buffer_release_callback_t = ());
ref_counter!(cef_v8value_t = ());
ref_counter!(cef_v8stack_trace_t = ());
ref_counter!(cef_v8stack_frame_t = ());
ref_counter!(cef_response_t = ());
ref_counter!(cef_resource_skip_callback_t = ());
ref_counter!(cef_resource_read_callback_t = ());
// ref_counter!(cef_scheme_registrar_t = ()); // doesn't seem to be ref-counted; investigate further as it also has base field
ref_counter!(_cef_scheme_handler_factory_t = ());
ref_counter!(cef_audio_handler_t = ());
ref_counter!(_cef_menu_model_t = ());
ref_counter!(_cef_run_context_menu_callback_t = ());
ref_counter!(cef_context_menu_handler_t = ());
ref_counter!(_cef_context_menu_params_t = ());
ref_counter!(_cef_file_dialog_callback_t = ());
ref_counter!(cef_dialog_handler_t = ());
ref_counter!(cef_display_handler_t = ());
ref_counter!(_cef_download_item_t = ());
ref_counter!(_cef_before_download_callback_t = ());
ref_counter!(_cef_download_item_callback_t = ());
ref_counter!(cef_download_handler_t = ());
ref_counter!(cef_drag_handler_t = ());
ref_counter!(cef_find_handler_t = ());
ref_counter!(cef_focus_handler_t = ());
ref_counter!(_cef_jsdialog_callback_t = ());
ref_counter!(cef_jsdialog_handler_t = ());
ref_counter!(cef_keyboard_handler_t = ());
ref_counter!(cef_life_span_handler_t = ());
ref_counter!(cef_accessibility_handler_t = ());
ref_counter!(cef_render_handler_t = ());
ref_counter!(cef_auth_callback_t = ());
ref_counter!(cef_request_callback_t = ());
ref_counter!(_cef_sslinfo_t = ());
ref_counter!(_cef_select_client_certificate_callback_t = ());
ref_counter!(cef_request_handler_t = ());
ref_counter!(cef_urlrequest_t = ());
ref_counter!(cef_web_plugin_info_t = ());
ref_counter!(cef_web_plugin_info_visitor_t = ());
ref_counter!(cef_web_plugin_unstable_callback_t = ());
ref_counter!(cef_register_cdm_callback_t = ());
