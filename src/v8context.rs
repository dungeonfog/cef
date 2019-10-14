use cef_sys::{cef_string_t, cef_v8context_t, cef_v8exception_t, cef_v8stack_trace_t, cef_v8value_t, cef_v8value_create_undefined, cef_v8value_create_null, cef_v8value_create_bool, cef_v8value_create_int, cef_v8value_create_uint, cef_v8value_create_double, cef_v8value_create_date, cef_time_from_doublet, cef_v8value_create_string, cef_v8accessor_t, cef_v8interceptor_t, cef_v8value_create_object, cef_v8value_create_array, cef_v8value_create_array_buffer, cef_v8array_buffer_release_callback_t, cef_v8value_create_function, cef_v8handler_t, cef_register_extension};
use std::{
    time::{SystemTime, SystemTimeError},
    convert::TryFrom,
    ptr::null_mut,
};

use crate::{
    client::Client,
    string::CefString,
    refcounted::RefCounted,
};

ref_counted_ptr! {
    pub struct V8Context<C: Client>(*mut cef_v8context_t);
}

impl<C> V8Context<C> where C: Client {
    /// Register a new V8 extension with the specified JavaScript extension code and
    /// handler. Functions implemented by the handler are prototyped using the
    /// keyword 'native'. The calling of a native function is restricted to the scope
    /// in which the prototype of the native function is defined. This function may
    /// only be called on the render process main thread.
    ///
    /// Example JavaScript extension code:
    /// ```js
    ///   // create the 'example' global object if it doesn't already exist.
    ///   if (!example)
    ///     example = {};
    ///   // create the 'example.test' global object if it doesn't already exist.
    ///   if (!example.test)
    ///     example.test = {};
    ///   (function() {
    ///     // Define the function 'example.test.myfunction'.
    ///     example.test.myfunction = function() {
    ///       // Call the handler closure with the function name 'MyFunction'
    ///       // and no arguments.
    ///       native function MyFunction();
    ///       return MyFunction();
    ///     };
    ///     // Define the getter function for parameter 'example.test.myparam'.
    ///     example.test.__defineGetter__('myparam', function() {
    ///       // Call the handler closure with the function name 'GetMyParam'
    ///       // and no arguments.
    ///       native function GetMyParam();
    ///       return GetMyParam();
    ///     });
    ///     // Define the setter function for parameter 'example.test.myparam'.
    ///     example.test.__defineSetter__('myparam', function(b) {
    ///       // Call the handler closure with the function name 'SetMyParam'
    ///       // and a single argument.
    ///       native function SetMyParam();
    ///       if(b) SetMyParam(b);
    ///     });
    ///
    ///     // Extension definitions can also contain normal JavaScript variables
    ///     // and functions.
    ///     var myint = 0;
    ///     example.test.increment = function() {
    ///       myint += 1;
    ///       return myint;
    ///     };
    ///   })();
    /// ```
    /// Example usage in the page:
    /// ```js
    ///   // Call the function.
    ///   example.test.myfunction();
    ///   // Set the parameter.
    ///   example.test.myparam = value;
    ///   // Get the parameter.
    ///   value = example.test.myparam;
    ///   // Call another function.
    ///   example.test.increment();
    /// ```
    pub fn register_extension(extension_name: &str, javascript_code: &str, handler: impl Fn(&str, &V8Value, &[&V8Value]) -> Result<V8Value, String> + 'static) {
        let name = CefString::new(extension_name);
        let js = CefString::new(javascript_code);
        unsafe {
            cef_register_extension(name.as_ptr(), js.as_ptr(), V8HandlerWrapper::new(handler));
        }
    }
}

ref_counted_ptr! {
    pub struct V8Exception(*mut cef_v8exception_t);
}

impl V8Exception {
    /// Returns the exception message.
    pub fn get_message(&self) -> String {
        "".to_owned()
    }
}

ref_counted_ptr! {
    pub struct V8StackTrace(*mut cef_v8stack_trace_t);
}

/// Structure representing a V8 value handle. V8 handles can only be accessed
/// from the thread on which they are created. Valid threads for creating a V8
/// handle include the render process main thread (TID_RENDERER) and WebWorker
/// threads. A task runner for posting tasks on the associated thread can be
/// retrieved via the [V8Context::get_task_runner] function.
pub struct V8Value(*mut cef_v8value_t);

impl V8Value {
    pub(crate) unsafe fn wrap(value: *mut cef_v8value_t) -> Self {
        V8Value(value)
    }
    pub(crate) fn as_ptr(&self) -> *const cef_v8value_t {
        self.0
    }
    pub(crate) fn into_raw(self) -> *mut cef_v8value_t {
        let result = self.0;
        self.0 = null_mut();
        result
    }
    /// Create a new V8Value object of type undefined.
    pub fn undefined() -> Self {
        V8Value(unsafe { cef_v8value_create_undefined() })
    }
    /// Create a new V8Value object of type null.
    pub fn null() -> Self {
        V8Value(unsafe { cef_v8value_create_null() })
    }
    /// Create a new V8Value object of type object with optional accessor
    /// and/or interceptor. This function should only be called from within the scope
    /// of a [RenderProcessHandler], [V8Handler] or [V8Accessor]
    /// callback, or in combination with calling [V8Context::enter] and [V8Context::exit] on a stored
    /// [V8Context] reference.
    pub fn new_object(accessor: Option<impl V8Accessor>, interceptor: Option<impl V8Interceptor>) -> Self {
        V8Value(unsafe {
            cef_v8value_create_object(accessor.map(V8AccessorWrapper::wrap).unwrap_or_else(null_mut), interceptor.map(V8InterceptorWrapper::wrap).unwrap_or_else(null_mut))
        })
    }
    /// Create a new V8Value object of type array with the specified `length`.
    /// If `length` is negative the returned array will have length 0. This function
    /// should only be called from within the scope of a
    /// [RenderProcessHandler], [V8Handler] or [V8Accessor] callback,
    /// or in combination with calling [V8Context::enter] and [V8Context::exit] on a stored V8Context
    /// reference.
    pub fn new_array(length: i32) -> Self {
        V8Value(unsafe {
            cef_v8value_create_array(length)
        })
    }
    /// Create a new V8Value object of type ArrayBuffer which wraps the
    /// provided `buffer` (without copying it). This function should only
    /// be called from within the scope of a [RenderProcessHandler], [V8Handler]
    /// or [V8Accessor] callback, or in combination with calling
    /// [V8Context::enter] and [V8Context::exit] on a stored [V8Context]
    /// reference.
    pub fn new_array_buffer(buffer: Vec<u8>) -> Self {
        let length = buffer.len();
        let capacity = buffer.capacity();
        let ptr = buffer.as_mut_ptr();
        std::mem::forget(buffer);
        V8Value(unsafe {
            cef_v8value_create_array_buffer(ptr, length, V8ArrayBufferReleaseCallbackWrapper::new(|ptr| {
                Vec::from_raw_parts(ptr, length, capacity);
            }))
        })
    }
    /// Create a new V8Value object of type function. This function
    /// should only be called from within the scope of a
    /// [RenderProcessHandler], [V8Handler] or [V8Accessor] callback,
    /// or in combination with calling [V8Context::enter] and [V8Context::exit] on a stored [V8Context]
    /// reference.
    pub fn new_function(name: &str, handler: impl Fn(&str, &V8Value, &[&V8Value]) -> Result<V8Value, String> + 'static) -> Self {
        let name = CefString::new(name);
        V8Value(unsafe {
            cef_v8value_create_function(name.as_ptr(), V8HandlerWrapper::new(handler))
        })
    }
}

impl Drop for V8Value {
    fn drop(&mut self) {
        if !self.0.is_null() {
            let release = self.0.base.release.unwrap();
            unsafe {
                release(&self.0.base);
            }
        }
    }
}

impl From<bool> for V8Value {
    /// Create a new V8Value object of type bool.
    fn from(value: bool) -> Self {
        V8Value(unsafe { cef_v8value_create_bool(value as i32) })
    }
}

impl From<i32> for V8Value {
    /// Create a new V8Value object of type int.
    fn from(value: i32) -> Self {
        V8Value(unsafe { cef_v8value_create_int(value) })
    }
}

impl From<u32> for V8Value {
    /// Create a new V8Value object of type unsigned int.
    fn from(value: u32) -> Self {
        V8Value(unsafe { cef_v8value_create_uint(value) })
    }
}

impl From<f64> for V8Value {
    /// Create a new V8Value object of type double.
    fn from(value: f64) -> Self {
        V8Value(unsafe { cef_v8value_create_double(value) })
    }
}

impl TryFrom<SystemTime> for V8Value {
    type Error = SystemTimeError;
    /// Create a new V8Value object of type Date. This function should only be
    /// called from within the scope of a [RenderProcessHandler],
    /// [V8Handler] or [V8AccessorHandler] callback, or in combination with calling
    /// [V8Context::enter] and [V8Context::exit] on a stored [V8Context] reference.
    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        let duration = value.duration_since(SystemTime::UNIX_EPOCH)?;
        let result = unsafe { std::mem::zeroed() };

        unsafe { cef_time_from_doublet(duration.as_secs_f64(), &mut result); } // This could fail in theory, but the actual implementation only returns 0 if the result is NULL
        Ok(V8Value(unsafe { cef_v8value_create_date(result) }))
    }
}

impl From<&str> for V8Value {
    /// Create a new V8Value object of type string.
    fn from(value: &str) -> Self {
        let cefstr = CefString::new(value);
        V8Value(unsafe { cef_v8value_create_string(cefstr.as_ref()) }) // cef_v8value_t takes ownership of this string by copying the pointer to the data and the dtor to its own structure and nulling out ours, so our Drop implementation won't do anything.
    }
}

/// Trait that should be implemented to handle V8 accessor calls. Accessor
/// identifiers are registered by calling [V8Value::set_value]. The
/// functions of this trait will be called on the thread associated with the
/// V8 accessor.
pub trait V8Accessor: 'static {
    /// Handle retrieval the accessor value identified by `name`. `object` is the
    /// receiver ('this' object) of the accessor. If retrieval succeeds return
    /// `Ok(retval)`. If retrieval fails return `Err(exception)` to be thrown 
    /// as an exception.
    fn get(&self, name: &str, object: &V8Value) -> Result<V8Value, String>;
    /// Handle assignment of the accessor value identified by `name`. `object` is
    /// the receiver ('this' object) of the accessor. `value` is the new value
    /// being assigned to the accessor. If assignment fails return `Err(exception)`
    /// to be thrown as an exception.
    fn set(&mut self, name: &str, object: &V8Value, value: &V8Value) -> Result<(), String>;
}

pub(crate) struct V8AccessorWrapper(Box<dyn V8Accessor>);

impl V8AccessorWrapper {
    pub(crate) fn new(
        delegate: Box<dyn V8Accessor>,
    ) -> *mut cef_v8accessor_t {
        let rc = RefCounted::new(
            cef_v8accessor_t {
                base: unsafe { std::mem::zeroed() },
                get: Some(Self::get),
                set: Some(Self::set),
            },
            delegate,
        );
        unsafe { &mut *rc }.get_cef()
    }
}

cef_callback_impl! {
    impl V8AccessorWrapper: cef_v8accessor_t {
        fn get(&mut self,
            name:      CefString     : *const cef_string_t,
            object:    V8Value       : *mut cef_v8value_t,
            retval:    &mut V8Value  : *mut *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name = unsafe { name.as_string().unwrap() };
            match self.0.get(&name, &object) {
                Ok(value) => {
                    (*retval) = value.into_raw();
                    1
                },
                Err(exception_str) => {
                    CefString::new(exception_str).move_to(exception);
                    0
                },
            }
        }
        fn set(&mut self,
            name     : CefString     : *const cef_string_t,
            object   : V8Value       : *mut cef_v8value_t,
            value    : V8Value       : *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) {
            let name = unsafe { name.as_string().unwrap() };
            if let Err(exception_str) = self.0.set(&name, &object, &value) {
                CefString::new(exception_str).move_to(exception);
                0
            } else {
                1
            }
        }
    }
}

/// Trait that should be implemented to handle V8 interceptor calls. The
/// functions of this trait will be called on the thread associated with the
/// V8 interceptor. Interceptor's named property handlers (with first argument of
/// type `&str`) are called when object is indexed by string. Indexed property
/// handlers (with first argument of type `i32`) are called when object is indexed
/// by integer.
pub trait V8Interceptor: 'static {
    /// Handle retrieval of the interceptor value identified by `name`. `object` is
    /// the receiver ('this' object) of the interceptor. If retrieval succeeds, return
    /// `Some(Ok(_))` containing the return value. If the requested value does not exist, return
    /// None. If retrieval fails, return `Some(Err(_))` containing
    /// the exception that will be thrown. If the property has an associated
    /// accessor, it will be called only if you return None.
    fn get_byname(&self, name: &str, object: &V8Value) -> Option<Result<V8Value, String>>;
    /// Handle retrieval of the interceptor value identified by `index`. `object` is
    /// the receiver ('this' object) of the interceptor. If retrieval succeeds, return
    /// `Some(Ok(_))` containing the return value. If the requested value does not exist, return
    /// None. If retrieval fails, return `Some(Err(_))` containing
    /// the exception that will be thrown. If the property has an associated
    /// accessor, it will be called only if you return None.
    fn get_byindex(&self, index: i32, object: &V8Value) -> Option<Result<V8Value, String>>;
    /// Handle assignment of the interceptor value identified by `name`. `object`
    /// is the receiver ('this' object) of the interceptor. `value` is the new
    /// value being assigned to the interceptor. If assignment fails, return
    /// `Err(_)` with the exception that will be thrown. This setter will always
    /// be called, even when the property has an associated accessor.
    fn set_byname(&mut self, name: &str, object: &V8Value, value: &V8Value) -> Result<(), String>;
    /// Handle assignment of the interceptor value identified by `index`. `object`
    /// is the receiver ('this' object) of the interceptor. `value` is the new
    /// value being assigned to the interceptor. If assignment fails, return
    /// `Err(_)` with the exception that will be thrown.
    fn set_byindex(&mut self, index: i32, object: &V8Value, value: &V8Value) -> Result<(), String>;
}

pub(crate) struct V8InterceptorWrapper(Box<dyn V8Interceptor>);

impl V8InterceptorWrapper {
    pub(crate) fn new(
        delegate: Box<dyn V8Interceptor>,
    ) -> *mut cef_v8interceptor_t {
        let rc = RefCounted::new(
            cef_v8interceptor_t {
                base: unsafe { std::mem::zeroed() },
                get_byname: Some(Self::get_byname),
                get_byindex: Some(Self::get_byindex),
                set_byname: Some(Self::set_byname),
                set_byindex: Some(Self::set_byindex),
            },
            delegate,
        );
        unsafe { &mut *rc }.get_cef()
    }
}

cef_callback_impl! {
    impl V8InterceptorWrapper: cef_v8interceptor_t {
        fn get_byname(&mut self,
            name:      CefString     : *const cef_string_t,
            object:    V8Value       : *mut cef_v8value_t,
            retval:    &mut V8Value  : *mut *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name = unsafe { name.as_string().unwrap() };
            if let Some(found) = self.0.get_byname(&name, &object) {
                match found {
                    Ok(value) => {
                        (*retval) = value.into_raw();
                        1
                    },
                    Err(exception_str) => {
                        CefString::new(exception_str).move_to(exception);
                        0
                    },
                }
            } else {
                0
            }
        }
        fn get_byindex(&mut self,
            index:     i32           : std::os::raw::c_int,
            object:    V8Value       : *mut cef_v8value_t,
            retval:    &mut V8Value  : *mut *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) -> std::os::raw::c_int {
            if let Some(found) = self.0.get_byindex(index, &object) {
                match found {
                    Ok(value) => {
                        (*retval) = value.into_raw();
                        1
                    },
                    Err(exception_str) => {
                        CefString::new(exception_str).move_to(exception);
                        0
                    },
                }
            } else {
                0
            }
        }
        fn set_byname(&mut self,
            name     : CefString     : *const cef_string_t,
            object   : V8Value       : *mut cef_v8value_t,
            value    : V8Value       : *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) {
            let name = unsafe { name.as_string().unwrap() };
            if let Err(exception_str) = self.0.set_byname(&name, &object, &value) {
                CefString::new(exception_str).move_to(exception);
                0
            } else {
                1
            }
        }
        fn set_byindex(&mut self,
            index    : i32           : std::os::raw::c_int,
            object   : V8Value       : *mut cef_v8value_t,
            value    : V8Value       : *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) {
            if let Err(exception_str) = self.0.set_byindex(index, &object, &value) {
                CefString::new(exception_str).move_to(exception);
                0
            } else {
                1
            }
        }
    }
}

struct V8ArrayBufferReleaseCallbackWrapper(Option<Box<dyn FnOnce(*mut u8) + 'static>>);

impl V8ArrayBufferReleaseCallbackWrapper {
    fn new(
        delegate: impl FnOnce(*mut u8) + 'static
    ) -> *mut cef_v8array_buffer_release_callback_t {
        let rc = RefCounted::new(
            cef_v8array_buffer_release_callback_t {
                base: unsafe { std::mem::zeroed() },
                release_buffer: Some(Self::release_buffer),
            },
            Box::new(delegate),
        );
        unsafe { &mut *rc }.get_cef()
    }
}

cef_callback_impl! {
    impl V8ArrayBufferReleaseCallbackWrapper: cef_v8array_buffer_release_callback_t {
        fn release_buffer(&mut self, buffer: *mut u8: *mut ::std::os::raw::c_void) {
            self.0.take()(buffer);
        }
    }
}

struct V8HandlerWrapper(Box<dyn Fn(&str, &V8Value, &[&V8Value]) -> Result<V8Value, String> + 'static>);

impl V8HandlerWrapper {
    fn new(
        delegate: impl Fn(&str, &V8Value, &[&V8Value]) -> Result<V8Value, String> + 'static
    ) -> *mut cef_v8handler_t {
        let rc = RefCounted::new(
            cef_v8handler_t {
                base: unsafe { std::mem::zeroed() },
                execute: Some(Self::execute),
            },
            Box::new(delegate),
        );
        unsafe { &mut *rc }.get_cef()
    }
}

cef_callback_impl! {
    impl V8HandlerWrapper: cef_v8handler_t {
        fn execute(&mut self,
            name           : CefString                 : *const cef_string_t,
            object         : V8Value                   : *mut cef_v8value_t,
            arguments_count: usize                     : usize,
            arguments      : *const *mut cef_v8value_t : *const *mut cef_v8value_t,
            retval         : &mut V8Value              : *mut *mut cef_v8value_t,
            exception      : *mut cef_string_t         : *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name = name.as_string().unwrap();
            let args = unsafe { std::slice::from_raw_parts(arguments, arguments_count).map(V8Value::wrap) }.collect();
            match self.0(&name, object, args) {
                Ok(value) => {
                    (*retval) = value;
                    1
                }
                Err(err) => {
                    CefString::new(err).move_to(exception);
                    0
                }
            }
        }
    }
}
