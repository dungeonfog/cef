use crate::{
    app::App,
    browser::Browser,
    browser_host::BrowserHost,
    callback::Callback,
    command_line::CommandLine,
    dom::{DOMDocument, DOMNode},
    drag::DragData,
    frame::Frame,
    image::Image,
    navigation::NavigationEntry,
    process::{ProcessId, ProcessMessage},
    request::{PostData, PostDataElement, Request},
    request_context::RequestContext,
    scheme_registrar::SchemeRegistrar,
    string::{CefString, CefStringList},
    url_request::URLRequestStatus,
    url_request::{AuthCallback, RequestCallback, Response, URLRequest},
    v8context::{V8Context, V8Exception, V8StackTrace},
    values::{DictionaryValue, ListValue, Value},
};
use std::{convert::TryFrom, os::raw::c_int};

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
        impl$(<$($generic $(: $bound)?),+>)? CToRustType for $Self {
            type CType = $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                <$Self>::from_ptr_unchecked(c_type)
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

macro_rules! owned_casts_from_unchecked {
    (impl for $Self:ty: $CType:ty) => {
        impl CToRustType for $Self {
            type CType = $CType;
            unsafe fn from_c_type(c_type: Self::CType) -> Self {
                Self::from_unchecked(c_type)
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

owned_casts!(impl for App = *mut cef_sys::cef_app_t);
owned_casts!(impl for Browser = *mut cef_sys::cef_browser_t);
owned_casts!(impl for BrowserHost = *mut cef_sys::cef_browser_host_t);
owned_casts!(impl for Callback = *mut cef_sys::cef_callback_t);
owned_casts!(impl for CommandLine = *mut cef_sys::cef_command_line_t);
owned_casts!(impl for DOMNode = *mut cef_sys::cef_domnode_t);
owned_casts!(impl for DOMDocument = *mut cef_sys::cef_domdocument_t);
owned_casts!(impl for DragData = *mut cef_sys::cef_drag_data_t);
owned_casts!(impl for Frame = *mut cef_sys::cef_frame_t);
owned_casts!(impl for Image = *mut cef_sys::cef_image_t);
owned_casts!(impl for NavigationEntry = *mut cef_sys::cef_navigation_entry_t);
owned_casts!(impl for ProcessMessage = *mut cef_sys::cef_process_message_t);
owned_casts!(impl for Request = *mut cef_sys::cef_request_t);
owned_casts!(impl for PostData = *mut cef_sys::cef_post_data_t);
owned_casts!(impl for PostDataElement = *mut cef_sys::cef_post_data_element_t);
owned_casts!(impl for URLRequest = *mut cef_sys::cef_urlrequest_t);
owned_casts!(impl for AuthCallback = *mut cef_sys::cef_auth_callback_t);
owned_casts!(impl for Response = *mut cef_sys::cef_response_t);
owned_casts!(impl for RequestCallback = *mut cef_sys::cef_request_callback_t);
owned_casts!(impl for V8Context = *mut cef_sys::cef_v8context_t);
owned_casts!(impl for V8Exception = *mut cef_sys::cef_v8exception_t);
owned_casts!(impl for V8StackTrace = *mut cef_sys::cef_v8stack_trace_t);
owned_casts!(impl for Value = *mut cef_sys::cef_value_t);
owned_casts!(impl for DictionaryValue = *mut cef_sys::cef_dictionary_value_t);
owned_casts!(impl for ListValue = *mut cef_sys::cef_list_value_t);
owned_casts!(impl for SchemeRegistrar = *mut cef_sys::cef_scheme_registrar_t);
owned_casts!(impl for RequestContext = *mut cef_sys::cef_request_context_t);
owned_casts!(impl for crate::v8context::V8Handler = *mut cef_sys::cef_v8handler_t);
owned_casts!(impl for crate::v8context::V8Value = *mut cef_sys::cef_v8value_t);
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
owned_casts_from_unchecked!(impl for URLRequestStatus: cef_sys::cef_urlrequest_status_t::Type);
owned_casts_from_unchecked!(impl for ProcessId: cef_sys::cef_process_id_t::Type);
owned_casts_from_unchecked!(impl for crate::load_handler::ErrorCode: cef_sys::cef_errorcode_t::Type);
owned_casts_from_unchecked!(impl for crate::request_context::PluginPolicy: cef_sys::cef_plugin_policy_t::Type);
impl CToRustType for bool {
    type CType = c_int;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        c_type != 0
    }
}

impl CToRustType for CefStringList {
    type CType = cef_sys::cef_string_list_t;
    unsafe fn from_c_type(c_type: Self::CType) -> Self {
        Self::from_raw(c_type)
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
        )+
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
                    this.inner($($field_name),*)
                }
            )+
        }
    };
}
