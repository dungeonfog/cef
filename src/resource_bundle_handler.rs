use cef_sys::{
    cef_resource_bundle_handler_t, cef_scale_factor_t, cef_string_t, cef_string_utf16_set,
};


use crate::{
    refcounted::{RefCounted},
    string::CefString,
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
    fn wrap(scale_factor: cef_scale_factor_t::Type) -> Option<Self> {
        match scale_factor {
            cef_scale_factor_t::SCALE_FACTOR_100P => Some(Self::Factor100p),
            cef_scale_factor_t::SCALE_FACTOR_125P => Some(Self::Factor125p),
            cef_scale_factor_t::SCALE_FACTOR_133P => Some(Self::Factor133p),
            cef_scale_factor_t::SCALE_FACTOR_140P => Some(Self::Factor140p),
            cef_scale_factor_t::SCALE_FACTOR_150P => Some(Self::Factor150p),
            cef_scale_factor_t::SCALE_FACTOR_180P => Some(Self::Factor180p),
            cef_scale_factor_t::SCALE_FACTOR_200P => Some(Self::Factor200p),
            cef_scale_factor_t::SCALE_FACTOR_250P => Some(Self::Factor250p),
            cef_scale_factor_t::SCALE_FACTOR_300P => Some(Self::Factor300p),
            _ => None,
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
    fn get_localized_string(&self, string_id: i32, string: &str) -> Option<String> {
        None
    }
    /// Retrieves the contents of the specified scale independent |resource_id|. If
    /// the value is found then it will be returned. If the value is not found then this function
    /// will return None.
    fn get_data_resource(&self, resource_id: i32) -> Option<Vec<u8>> {
        None
    }
    /// Retrieves the contents of the specified |resource_id| nearest the scale
    /// factor |scale_factor|. Use a |scale_factor| value of None for
    /// scale independent resources or call `get_data_resource` instead. If the value
    /// is found then it will be returned. If the value is not found then this function will
    /// return None.
    fn get_data_resource_for_scale(
        &self,
        resource_id: i32,
        scale_factor: Option<ScaleFactor>,
    ) -> Option<Vec<u8>> {
        None
    }
}

pub struct ResourceBundleHandlerWrapper {}

impl ResourceBundleHandlerWrapper {
    pub(crate) fn new(
        delegate: Box<dyn ResourceBundleHandler>,
    ) -> *mut RefCounted<cef_resource_bundle_handler_t> {
        RefCounted::new(
            cef_resource_bundle_handler_t {
                base: unsafe { std::mem::zeroed() },
                get_localized_string: Some(Self::get_localized_string),
                get_data_resource: Some(Self::get_data_resource),
                get_data_resource_for_scale: Some(Self::get_data_resource_for_scale),
            },
            delegate,
        )
    }

    extern "C" fn get_localized_string(
        self_: *mut cef_resource_bundle_handler_t,
        string_id: std::os::raw::c_int,
        string: *mut cef_string_t,
    ) -> std::os::raw::c_int {
        let this = unsafe { RefCounted::<cef_resource_bundle_handler_t>::make_temp(self_) };
        match this.get_localized_string(string_id, &CefString::copy_raw_to_string(string).unwrap())
        {
            None => 0,
            Some(rstr) => {
                let utf16: Vec<u16> = rstr.encode_utf16().collect();
                unsafe {
                    cef_string_utf16_set(
                        utf16.as_ptr(),
                        utf16.len() * std::mem::size_of::<u16>(),
                        string,
                        1,
                    );
                }
                1
            }
        }
    }

    extern "C" fn get_data_resource(
        self_: *mut cef_resource_bundle_handler_t,
        resource_id: std::os::raw::c_int,
        data: *mut *mut std::os::raw::c_void,
        data_size: *mut usize,
    ) -> std::os::raw::c_int {
        let this = unsafe { RefCounted::<cef_resource_bundle_handler_t>::make_temp(self_) };
        match this.get_data_resource(resource_id) {
            None => 0,
            Some(bytes) => {
                unsafe {
                    (*data_size) = bytes.len();
                    (*data) = Box::into_raw(bytes.into_boxed_slice()) as *mut std::os::raw::c_void;
                }
                1
            }
        }
    }

    extern "C" fn get_data_resource_for_scale(
        self_: *mut cef_resource_bundle_handler_t,
        resource_id: std::os::raw::c_int,
        scale_factor: cef_scale_factor_t::Type,
        data: *mut *mut std::os::raw::c_void,
        data_size: *mut usize,
    ) -> std::os::raw::c_int {
        let this = unsafe { RefCounted::<cef_resource_bundle_handler_t>::make_temp(self_) };
        match this.get_data_resource_for_scale(resource_id, ScaleFactor::wrap(scale_factor)) {
            None => 0,
            Some(bytes) => {
                unsafe {
                    (*data_size) = bytes.len();
                    (*data) = Box::into_raw(bytes.into_boxed_slice()) as *mut std::os::raw::c_void;
                }
                1
            }
        }
    }
}
