use crate::{
    app::App,
    url_request::URLRequestStatus,
    browser::Browser,
    browser_host::BrowserHost,
    callback::Callback,
    command_line::CommandLine,
    dom::{DOMNode, DOMDocument},
    drag::DragData,
    frame::Frame,
    image::Image,
    navigation::NavigationEntry,
    process::ProcessMessage,
    request::{Request, PostData, PostDataElement},
    url_request::{URLRequest, AuthCallback, Response, RequestCallback},
    v8context::{V8Context, V8Exception, V8StackTrace},
    values::{Value, DictionaryValue, ListValue},
    string::CefString,
};

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

pub trait CTypeToOwned {
    type CType;
    type Owned;
    unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned;
}

pub trait RefMutTo<'a, O> {
    fn from_owned(b: &'a mut Self) -> O;
}

impl<'a, T> CTypeToOwned for Option<&'a T>
where
    &'a T: CTypeToOwned,
    <&'a T as CTypeToOwned>::CType: Pointer,
{
    type CType = <&'a T as CTypeToOwned>::CType;
    type Owned = Option<<&'a T as CTypeToOwned>::Owned>;

    unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
        if c_type.is_null() {
            None
        } else {
            Some(<&'a T as CTypeToOwned>::from_c_type(c_type))
        }
    }
}
impl<'a, T> RefMutTo<'a, Option<&'a T>> for Option<T>
where
    &'a T: CTypeToOwned<Owned = T>,
{
    fn from_owned(b: &'a mut Option<T>) -> Option<&'a T> {
        b.as_ref()
    }
}
impl<'a, T> RefMutTo<'a, Option<&'a T>> for Option<&'a T>
where
    &'a T: CTypeToOwned<Owned = T>,
{
    fn from_owned(b: &'a mut Option<&'a T>) -> Option<&'a T> {
        *b
    }
}

impl<'a, T> CTypeToOwned for Option<&'a mut T>
where
    &'a mut T: CTypeToOwned,
    <&'a mut T as CTypeToOwned>::CType: Pointer,
{
    type CType = <&'a mut T as CTypeToOwned>::CType;
    type Owned = Option<<&'a mut T as CTypeToOwned>::Owned>;

    unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
        if c_type.is_null() {
            None
        } else {
            Some(<&'a mut T as CTypeToOwned>::from_c_type(c_type))
        }
    }
}
impl<'a, T> RefMutTo<'a, Option<&'a mut T>> for Option<T>
where
    &'a mut T: CTypeToOwned,
{
    fn from_owned(b: &'a mut Option<T>) -> Option<&'a mut T> {
        b.as_mut()
    }
}
impl<'a, T> RefMutTo<'a, Option<&'a mut T>> for Option<&'a mut T>
where
    &'a mut T: CTypeToOwned,
{
    fn from_owned(b: &'a mut Option<&'a mut T>) -> Option<&'a mut T> {
        Option::<&mut &mut T>::from(b).map(|b| &mut **b)
    }
}

macro_rules! owned_casts {
    (impl<'a> for &'a $Self:ty = $CType:ty) => {
        impl<'a> CTypeToOwned for &'a $Self {
            type CType = $CType;
            type Owned = $Self;
            unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
                <$Self>::from_ptr_unchecked(c_type)
            }
        }
        impl<'a> RefMutTo<'a, &'a $Self> for <&'a $Self as CTypeToOwned>::Owned {
            fn from_owned(b: &'a mut $Self) -> &'a $Self {
                b
            }
        }
    };
}

macro_rules! owned_casts_no_transform {
    (impl for $Self:ty) => {
        impl<'a> CTypeToOwned for $Self {
            type CType = $Self;
            type Owned = $Self;
            unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
                c_type
            }
        }
        impl<'a> CTypeToOwned for &'a $Self {
            type CType = *mut $Self;
            type Owned = $Self;
            unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
                *c_type
            }
        }
        impl<'a> CTypeToOwned for &'a mut $Self {
            type CType = *mut $Self;
            type Owned = &'a mut $Self;
            unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
                &mut *c_type
            }
        }
        impl<'a> RefMutTo<'a, &'a $Self> for $Self {
            fn from_owned(b: &'a mut $Self) -> &'a $Self {
                b
            }
        }
        impl<'a> RefMutTo<'a, &'a mut $Self> for $Self {
            fn from_owned(b: &'a mut $Self) -> &'a mut $Self {
                b
            }
        }
        impl<'a> RefMutTo<'a, $Self> for $Self {
            fn from_owned(b: &'a mut $Self) -> $Self {
                *b
            }
        }
    };
}

owned_casts!(impl<'a> for &'a App = *mut cef_sys::cef_app_t);
owned_casts!(impl<'a> for &'a Browser = *mut cef_sys::cef_browser_t);
owned_casts!(impl<'a> for &'a BrowserHost = *mut cef_sys::cef_browser_host_t);
owned_casts!(impl<'a> for &'a Callback = *mut cef_sys::cef_callback_t);
owned_casts!(impl<'a> for &'a CommandLine = *mut cef_sys::cef_command_line_t);
owned_casts!(impl<'a> for &'a DOMNode = *mut cef_sys::cef_domnode_t);
owned_casts!(impl<'a> for &'a DOMDocument = *mut cef_sys::cef_domdocument_t);
owned_casts!(impl<'a> for &'a DragData = *mut cef_sys::cef_drag_data_t);
owned_casts!(impl<'a> for &'a Frame = *mut cef_sys::cef_frame_t);
owned_casts!(impl<'a> for &'a Image = *mut cef_sys::cef_image_t);
owned_casts!(impl<'a> for &'a NavigationEntry = *mut cef_sys::cef_navigation_entry_t);
owned_casts!(impl<'a> for &'a ProcessMessage = *mut cef_sys::cef_process_message_t);
owned_casts!(impl<'a> for &'a Request = *mut cef_sys::cef_request_t);
owned_casts!(impl<'a> for &'a PostData = *mut cef_sys::cef_post_data_t);
owned_casts!(impl<'a> for &'a PostDataElement = *mut cef_sys::cef_post_data_element_t);
owned_casts!(impl<'a> for &'a URLRequest = *mut cef_sys::cef_urlrequest_t);
owned_casts!(impl<'a> for &'a AuthCallback = *mut cef_sys::cef_auth_callback_t);
owned_casts!(impl<'a> for &'a Response = *mut cef_sys::cef_response_t);
owned_casts!(impl<'a> for &'a RequestCallback = *mut cef_sys::cef_request_callback_t);
owned_casts!(impl<'a> for &'a V8Context = *mut cef_sys::cef_v8context_t);
owned_casts!(impl<'a> for &'a V8Exception = *mut cef_sys::cef_v8exception_t);
owned_casts!(impl<'a> for &'a V8StackTrace = *mut cef_sys::cef_v8stack_trace_t);
owned_casts!(impl<'a> for &'a Value = *mut cef_sys::cef_value_t);
owned_casts!(impl<'a> for &'a DictionaryValue = *mut cef_sys::cef_dictionary_value_t);
owned_casts!(impl<'a> for &'a ListValue = *mut cef_sys::cef_list_value_t);
owned_casts_no_transform!(impl for i8);
owned_casts_no_transform!(impl for i16);
owned_casts_no_transform!(impl for i32);
owned_casts_no_transform!(impl for i64);
owned_casts_no_transform!(impl for u8);
owned_casts_no_transform!(impl for u16);
owned_casts_no_transform!(impl for u32);
owned_casts_no_transform!(impl for u64);

impl<'a> CTypeToOwned for &'a mut CefString {
    type CType = *mut cef_sys::cef_string_t;
    type Owned = &'a mut CefString;
    unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
        CefString::from_mut_ptr(c_type)
    }
}
impl<'a> RefMutTo<'a, &'a mut CefString> for &'a mut CefString {
    fn from_owned(b: &'a mut &'a mut CefString) -> Self {
        *b
    }
}

impl<'a> CTypeToOwned for URLRequestStatus {
    type CType = cef_sys::cef_urlrequest_status_t::Type;
    type Owned = URLRequestStatus;
    unsafe fn from_c_type(c_type: Self::CType) -> Self::Owned {
        unsafe { Self::Owned::from_unchecked(c_type) }
    }
}
impl<'a> RefMutTo<'a, URLRequestStatus> for URLRequestStatus {
    fn from_owned(b: &'a mut URLRequestStatus) -> Self {
        *b
    }
}

macro_rules! cef_callback_impl {
    (impl $RefCounted:ty: $CType:ty {
        $(fn $fn_name:ident(&mut $self:tt, $($field_name:ident: $field_ty:ty: $c_ty:ty),+ $(,)?) $(-> $ret:ty)? $body:block)+
    }) => {
        impl $RefCounted {
            $(
                extern "C" fn $fn_name(self_: *mut $CType, $($field_name: $c_ty),+) $(-> $ret)? {
                    trait Impl {
                        fn inner(&mut $self, $($field_name: $field_ty),+) $(-> $ret)?;
                    }
                    impl Impl for $RefCounted {
                        #[inline(always)]
                        fn inner(&mut $self, $($field_name: $field_ty),+) $(-> $ret)? $body
                    }
                    let mut this = unsafe { RefCounted::<$CType>::make_temp(self_) };
                    $(
                        let mut $field_name: <$field_ty as crate::extern_callback_helpers::CTypeToOwned>::Owned = unsafe{ <$field_ty as crate::extern_callback_helpers::CTypeToOwned>::from_c_type($field_name) };
                    )+
                    $(
                        let $field_name: $field_ty = <<$field_ty as crate::extern_callback_helpers::CTypeToOwned>::Owned as crate::extern_callback_helpers::RefMutTo<$field_ty>>::from_owned(&mut $field_name);
                    )+
                    this.inner($($field_name),+)
                }
            )+
        }
    };
}
