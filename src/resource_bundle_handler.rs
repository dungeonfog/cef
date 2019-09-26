use cef_sys::{cef_resource_bundle_handler_t, cef_string_userfree_utf16_alloc, cef_string_t, cef_string_utf8_to_utf16, cef_scale_factor_t, cef_string_utf16_set, cef_base_ref_counted_t};
use std::{
    ptr::null_mut,
    sync::Arc,
    convert::TryFrom,
};

use crate::{
    string::CefString,
    refcounted::{RefCounted, RefCounter},
    ptr_hash::Hashed,
    reference,
    PackResource, PackString,
};

/// Supported UI scale factors for the platform. None is used for
/// density independent resources such as string, html/js files or an image that
/// can be used for any scale factors (such as wallpapers).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ScaleFactor {
    Factor100p,
    Factor125p,
    Factor133p,
    Factor140p,
    Factor150p,
    Factor180p,
    Factor200p,
    Factor250p,
    Factor300p,
}

impl ScaleFactor {
    fn wrap(scale_factor: cef_scale_factor_t) -> Option<Self> {
        match scale_factor {
            cef_scale_factor_t::SCALE_FACTOR_NONE => None,
            cef_scale_factor_t::SCALE_FACTOR_100P => Some(Self::Factor100p),
            cef_scale_factor_t::SCALE_FACTOR_125P => Some(Self::Factor125p),
            cef_scale_factor_t::SCALE_FACTOR_133P => Some(Self::Factor133p),
            cef_scale_factor_t::SCALE_FACTOR_140P => Some(Self::Factor140p),
            cef_scale_factor_t::SCALE_FACTOR_150P => Some(Self::Factor150p),
            cef_scale_factor_t::SCALE_FACTOR_180P => Some(Self::Factor180p),
            cef_scale_factor_t::SCALE_FACTOR_200P => Some(Self::Factor200p),
            cef_scale_factor_t::SCALE_FACTOR_250P => Some(Self::Factor250p),
            cef_scale_factor_t::SCALE_FACTOR_300P => Some(Self::Factor300p),
        }
    }
}

/// Trait used for retrieving resources from the resource bundle (*.pak)
/// files loaded by CEF during startup or via the cef_resource_bundle_handler
/// returned from cef_app_t::GetResourceBundleHandler. See CefSettings for
/// additional options related to resource bundle loading. The functions of this
/// structure may be called on any thread unless otherwise indicated.
pub trait ResourceBundleHandler: Send + Sync {
    /// Called to retrieve a localized translation for the specified |string_id|.
    /// To provide the translation return the translation string.
    /// To use the default translation return None.
    fn get_localized_string(&self, string_id: PackString, string: &str) -> Option<String> { None }
    /// Retrieves the contents of the specified scale independent |resource_id|. If
    /// the value is found then it will be returned. If the value is not found then this function
    /// will return None.
    fn get_data_resource(&self, resource_id: PackResource) -> Option<Vec<u8>> { None }
    /// Retrieves the contents of the specified |resource_id| nearest the scale
    /// factor |scale_factor|. Use a |scale_factor| value of None for
    /// scale independent resources or call `get_data_resource` instead. If the value
    /// is found then it will be returned. If the value is not found then this function will
    /// return None.
    fn get_data_resource_for_scale(&self, resource_id: PackResource, scale_factor: Option<ScaleFactor>) -> Option<Vec<u8>> { None }
}

pub struct ResourceBundleHandlerWrapper {}

impl RefCounter for cef_resource_bundle_handler_t {
    type Wrapper = RefCounted<Self, Box<dyn ResourceBundleHandler>>;
    fn set_base(&mut self, base: cef_base_ref_counted_t) {
        self.base = base;
    }
}

impl ResourceBundleHandlerWrapper {
    pub(crate) fn new(delegate: Box<dyn ResourceBundleHandler>) -> *mut <cef_resource_bundle_handler_t as RefCounter>::Wrapper {
        RefCounted::new(cef_resource_bundle_handler_t {
            get_localized_string: Some(Self::get_localized_string),
            get_data_resource: Some(Self::get_data_resource),
            get_data_resource_for_scale: Some(Self::get_data_resource_for_scale),
            ..Default::default()
        }, delegate)
    }

    extern "C" fn get_localized_string(self_: *mut cef_resource_bundle_handler_t, string_id: std::os::raw::c_int, string: *mut cef_string_t) -> std::os::raw::c_int {
        if let Ok(string_id) = PackString::try_from(string_id) {
            let this = unsafe { <cef_resource_bundle_handler_t as RefCounter>::Wrapper::make_temp(self_) };
            match this.get_localized_string(string_id, &CefString::copy_raw_to_string(string).unwrap()) {
                None => 0,
                Some(rstr) => {
                    let utf16: Vec<u16> = rstr.encode_utf16().collect();
                    unsafe { cef_string_utf16_set(utf16.as_ptr(), utf16.len() * std::mem::size_of::<u16>(), string, 1); }
                    1
                }
            }
        } else { 0 }
    }

    extern "C" fn get_data_resource(self_: *mut cef_resource_bundle_handler_t, resource_id: std::os::raw::c_int, data: *mut *mut std::os::raw::c_void, data_size: *mut usize) -> std::os::raw::c_int {
        if let Ok(resource_id) = PackResource::try_from(resource_id) {
            let this = unsafe { <cef_resource_bundle_handler_t as RefCounter>::Wrapper::make_temp(self_) };
            match this.get_data_resource(resource_id) {
                None => 0,
                Some(bytes) => {
                    unsafe {
                        (*data_size) = bytes.len();
                        (*data) = Box::into_raw(bytes.into_boxed_slice()) as *mut std::os::raw::c_void;
                    }
                    1
                },
            }
        } else { 0 }
    }

    extern "C" fn get_data_resource_for_scale(self_: *mut cef_resource_bundle_handler_t, resource_id: std::os::raw::c_int, scale_factor: cef_scale_factor_t, data: *mut *mut std::os::raw::c_void, data_size: *mut usize) -> std::os::raw::c_int {
        if let Ok(resource_id) = PackResource::try_from(resource_id) {
            let this = unsafe { <cef_resource_bundle_handler_t as RefCounter>::Wrapper::make_temp(self_) };
            match this.get_data_resource_for_scale(resource_id, ScaleFactor::wrap(scale_factor)) {
                None => 0,
                Some(bytes) => {
                    unsafe {
                        (*data_size) = bytes.len();
                        (*data) = Box::into_raw(bytes.into_boxed_slice()) as *mut std::os::raw::c_void;
                    }
                    1
                },
            }
        } else { 0 }
    }
}
