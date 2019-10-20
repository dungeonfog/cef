use crate::refcounted::RefCountedPtr;

pub trait IsSame {
    fn is_same(&self, other: Self) -> bool;
}

macro_rules! is_same {
    ($cef:ident) => {
        impl IsSame for RefCountedPtr<cef_sys::$cef> {
            fn is_same(&self, other: Self) -> bool {
                unsafe { (self.is_same.unwrap())(self.as_ptr(), other.into_raw()) != 0 }
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

pub trait IsValid {
    fn is_valid(&self) -> bool;
}

macro_rules! is_valid {
    ($cef:ident) => {
        impl IsValid for RefCountedPtr<cef_sys::$cef> {
            fn is_valid(&self) -> bool {
                self.is_valid
                    .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
                    .unwrap_or(false)
            }
        }
    };
}

is_valid!(_cef_navigation_entry_t);
