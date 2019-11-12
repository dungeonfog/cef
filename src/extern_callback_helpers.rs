use crate::{
    process::{ProcessId},
    string::{CefString, CefStringList},
    url_request::URLRequestStatus,
};
use std::{convert::TryFrom, mem::ManuallyDrop, os::raw::c_int};

pub trait Pointer: Copy {
    fn is_null(self) -> bool;
}

impl<T> Pointer for *const T {
    fn is_null(self) -> bool {
        self.is_null()
    }
}

impl<T> Pointer for *mut T {
    fn is_null(self) -> bool {
        self.is_null()
    }
}

pub trait CToRustType {
    type CType;
    unsafe fn from_c_type(c_type: Self::CType) -> Self;
}

impl<T> CToRustType for Option<T>
where
    T: CToRustType,
    <T as CToRustType>::CType: Pointer,
{
    type CType = <T as CToRustType>::CType;

    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        if c_type.is_null() {
            None
        } else {
            Some(<T as CToRustType>::from_c_type(c_type))
        }
    }
}

macro_rules! owned_casts {
    (impl$(<$($generic:ident $(: $bound:path)?),+>)? for $Self:ty = $CType:ty) => {
        impl$(<$($generic $(: $bound)?),+>)? crate::extern_callback_helpers::CToRustType for $Self {
            type CType = $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                <$Self>::from_ptr_unchecked(c_type)
            }
        }
        impl$(<$($generic $(: $bound)?),+>)? crate::extern_callback_helpers::CToRustType for &mut $Self {
            type CType = *mut $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                <$Self>::from_ptr_ptr(c_type)
            }
        }
    };
}

macro_rules! owned_casts_no_transform {
    (impl for $Self:ty) => {
        impl CToRustType for $Self {
            type CType = $Self;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                c_type
            }
        }
        impl<'a> CToRustType for &'a $Self {
            type CType = *mut $Self;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                &*c_type
            }
        }
        impl<'a> CToRustType for &'a mut $Self {
            type CType = *mut $Self;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                &mut *c_type
            }
        }
    };
}

macro_rules! owned_casts_from {
    (impl for $Self:ty: $CType:ty) => {
        impl CToRustType for $Self {
            type CType = $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                Self::from(c_type)
            }
        }
        impl<'a> CToRustType for &'a $Self {
            type CType = *const $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                assert_eq!(
                    std::mem::size_of::<$CType>(),
                    std::mem::size_of::<$Self>()
                );
                &*(c_type as *const $Self)
            }
        }
        impl<'a> CToRustType for &'a mut $Self {
            type CType = *mut $CType;
            #[inline(always)]
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                assert_eq!(
                    std::mem::size_of::<$CType>(),
                    std::mem::size_of::<$Self>()
                );
                &mut *(c_type as *mut $Self)
            }
        }
    };
}

macro_rules! owned_casts_from_unchecked {
    (impl for $Self:ty: $CType:ty) => {
        impl CToRustType for $Self {
            type CType = $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                Self::from_unchecked(c_type)
            }
        }
        impl<'a> CToRustType for &'a $Self {
            type CType = *const $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                assert_eq!(
                    std::mem::size_of::<Self::CType>(),
                    std::mem::size_of::<Self>()
                );
                &*(c_type as *const Self)
            }
        }
        impl<'a> CToRustType for &'a mut $Self {
            type CType = *mut $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                assert_eq!(
                    std::mem::size_of::<Self::CType>(),
                    std::mem::size_of::<Self>()
                );
                &mut *(c_type as *mut Self)
            }
        }
    };
}

macro_rules! owned_casts_from_flags_unchecked {
    (impl for $Self:ty: $CType:ty) => {
        impl CToRustType for $Self {
            type CType = $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                Self::from_unchecked(c_type.0)
            }
        }
        impl<'a> CToRustType for &'a $Self {
            type CType = *mut $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                assert_eq!(
                    std::mem::size_of::<Self::CType>(),
                    std::mem::size_of::<Self>()
                );
                &*(c_type as *mut Self)
            }
        }
        impl<'a> CToRustType for &'a mut $Self {
            type CType = *mut $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                assert_eq!(
                    std::mem::size_of::<Self::CType>(),
                    std::mem::size_of::<Self>()
                );
                &mut *(c_type as *mut Self)
            }
        }
    };
}

owned_casts_no_transform!(impl for i8);
owned_casts_no_transform!(impl for i16);
owned_casts_no_transform!(impl for i32);
owned_casts_no_transform!(impl for i64);
owned_casts_no_transform!(impl for isize);
owned_casts_no_transform!(impl for u8);
owned_casts_no_transform!(impl for u16);
owned_casts_no_transform!(impl for u32);
owned_casts_no_transform!(impl for u64);
owned_casts_no_transform!(impl for usize);
owned_casts_no_transform!(impl for f32);
owned_casts_no_transform!(impl for f64);
owned_casts_from_unchecked!(impl for URLRequestStatus: cef_sys::cef_urlrequest_status_t::Type);
owned_casts_from_unchecked!(impl for ProcessId: cef_sys::cef_process_id_t::Type);
owned_casts_from_unchecked!(impl for crate::load_handler::ErrorCode: cef_sys::cef_errorcode_t::Type);
owned_casts_from_unchecked!(impl for crate::request_context::PluginPolicy: cef_sys::cef_plugin_policy_t::Type);
owned_casts_from_unchecked!(impl for crate::request_handler::WindowOpenDisposition: cef_sys::cef_window_open_disposition_t::Type);
owned_casts_from_unchecked!(impl for crate::browser_host::PaintElementType: cef_sys::cef_paint_element_type_t::Type);
owned_casts_from_unchecked!(impl for crate::client::render_handler::TextInputMode: cef_sys::cef_text_input_mode_t::Type);
owned_casts_from_unchecked!(impl for crate::client::context_menu_handler::MenuColorType: cef_sys::cef_menu_color_type_t::Type);
owned_casts_from_unchecked!(impl for crate::client::focus_handler::FocusSource: cef_sys::cef_focus_source_t::Type);
owned_casts_from_unchecked!(impl for crate::client::js_dialog_handler::JsDialogType: cef_sys::cef_jsdialog_type_t::Type);
owned_casts_from_unchecked!(impl for crate::settings::LogSeverity: cef_sys::cef_log_severity_t::Type);
owned_casts_from_flags_unchecked!(impl for crate::drag::DragOperation: cef_sys::cef_drag_operations_mask_t);
owned_casts_from_flags_unchecked!(impl for crate::events::EventFlags: cef_sys::cef_event_flags_t);
owned_casts_from_unchecked!(impl for crate::color::Color: cef_sys::cef_color_t);
owned_casts_from_unchecked!(impl for crate::client::context_menu_handler::CommandId: i32);
owned_casts_from_unchecked!(impl for crate::client::context_menu_handler::GroupId: i32);
owned_casts_from!(impl for crate::values::Rect: cef_sys::cef_rect_t);
owned_casts_from!(impl for crate::values::Point: cef_sys::cef_point_t);
owned_casts_from!(impl for crate::values::Size: cef_sys::cef_size_t);
owned_casts_from!(impl for crate::values::Range: cef_sys::cef_range_t);
owned_casts_from!(impl for crate::events::KeyEvent: cef_sys::cef_key_event_t);
impl CToRustType for bool {
    type CType = c_int;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        c_type != 0
    }
}

impl CToRustType for ManuallyDrop<CefStringList> {
    type CType = cef_sys::cef_string_list_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        CefStringList::from_raw(c_type)
    }
}

impl<'a> CToRustType for &'a CefString {
    type CType = *const cef_sys::cef_string_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        CefString::from_ptr_unchecked(c_type)
    }
}

impl<'a> CToRustType for &'a mut CefString {
    type CType = *mut cef_sys::cef_string_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        CefString::from_mut_ptr_unchecked(c_type)
    }
}

impl CToRustType for crate::load_handler::TransitionType {
    type CType = cef_sys::cef_transition_type_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        Self::try_from(c_type.0).unwrap()
    }
}

impl CToRustType for crate::web_plugin::WebPluginInfo {
    type CType = *mut cef_sys::cef_web_plugin_info_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        Self::new(c_type)
    }
}

impl CToRustType for Option<crate::resource_bundle_handler::ScaleFactor> {
    type CType = cef_sys::cef_scale_factor_t::Type;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        crate::resource_bundle_handler::ScaleFactor::wrap(c_type)
    }
}
impl CToRustType for crate::cookie::Cookie {
    type CType = *const cef_sys::cef_cookie_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        Self::new(c_type)
    }
}
impl CToRustType for crate::client::life_span_handler::PopupFeatures {
    type CType = *const cef_sys::_cef_popup_features_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        Self::new(&*c_type)
    }
}
impl CToRustType for crate::window::WindowInfo {
    type CType = *const cef_sys::cef_window_info_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        Self::from(&*c_type)
    }
}
impl CToRustType for crate::file_dialog::FileDialogMode {
    type CType = cef_sys::cef_file_dialog_mode_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        Self::try_from(c_type).unwrap()
    }
}

impl<'a> CToRustType for &mut cef_sys::cef_screen_info_t {
    type CType = *mut cef_sys::cef_screen_info_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        &mut *c_type
    }
}

impl<'a, T> CToRustType for &mut *mut T {
    type CType = *mut *mut T;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        &mut *c_type
    }
}
impl<T> CToRustType for *const T {
    type CType = *const T;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        c_type
    }
}
impl<T> CToRustType for *mut T {
    type CType = *mut T;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        c_type
    }
}

macro_rules! cef_callback_impl {
    (impl$(<$($generic:ident $(: $bound:path)?),+>)? for $RefCounted:ty: $CType:ty {
        $(
            $(#[$meta:meta])*
            fn $fn_name:ident$(<$($igeneric:ident $(: $ibound:path)?),+>)?(&$self:ident $(, $field_name:ident: $field_ty:ty: $c_ty:ty)* $(,)?) $(-> $ret:ty)? $body:block
        )*
    }) => {
        impl$(<$($generic $(: $bound)?),+>)? $RefCounted {
            $(
                $(#[$meta])*
                extern "C" fn $fn_name(self_: *mut $CType, $($field_name: $c_ty),*) $(-> $ret)? {
                    trait Impl$(<$($igeneric $(: $ibound)?),+>)? {
                        fn inner(&$self, $($field_name: $field_ty),*) $(-> $ret)?;
                    }
                    impl$(<$($igeneric $(: $ibound)?),+>)? Impl$(<$($igeneric),+>)? for $RefCounted {
                        #[inline(always)]
                        fn inner(&$self, $($field_name: $field_ty),*) $(-> $ret)? $body
                    }
                    let this = unsafe { crate::refcounted::RefCounted::<$RefCounted>::wrapper(self_) };
                    $(
                        let $field_name: $field_ty = unsafe{ <$field_ty as crate::extern_callback_helpers::CToRustType>::from_c_type($field_name) };
                    )*
                    let ret = this.inner($($field_name),*);
                    ret
                }
            )*
        }
    };
}
