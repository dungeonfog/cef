use cef_sys::{cef_string_t, cef_v8context_t, cef_v8exception_t, cef_v8stack_trace_t, cef_v8value_t, cef_v8value_create_undefined, cef_v8value_create_null, cef_v8value_create_bool, cef_v8value_create_int, cef_v8value_create_uint, cef_v8value_create_double, cef_v8value_create_date, cef_time_from_doublet, cef_v8value_create_string, cef_v8accessor_t, cef_v8interceptor_t, cef_v8value_create_object, cef_v8value_create_array, cef_v8value_create_array_buffer, cef_v8array_buffer_release_callback_t, cef_v8value_create_function, cef_v8handler_t, cef_register_extension, cef_v8_propertyattribute_t, cef_v8_accesscontrol_t, cef_base_ref_counted_t};
use std::{
    time::{SystemTime, SystemTimeError},
    convert::TryFrom,
    ptr::null_mut,
    collections::HashSet,
    any::Any,
    sync::Arc,
};

use crate::{
    client::Client,
    string::{CefString, CefStringList},
    refcounted::{RefCounted, RefCounterWrapped},
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

/// V8 property attribute values.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum V8PropertyAttribute {
    /// Not writeable
    ReadOnly = cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_READONLY,
    /// Not enumerable
    DontEnum = cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_DONTENUM,
    /// Not configurable
    DontDelete = cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_DONTDELETE,
}

impl V8PropertyAttribute {
    pub(crate) fn as_mask<'a, I: 'a + Iterator<Item = &'a Self>>(attributes: I) -> cef_v8_propertyattribute_t {
        cef_v8_propertyattribute_t(attributes.fold(0, |mask, attr| mask | attr.0))
    }
    pub(crate) fn as_vec(mask: cef_v8_propertyattribute_t) -> HashSet<Self> {
        [
            V8PropertyAttribute::ReadOnly,
            V8PropertyAttribute::DontEnum,
            V8PropertyAttribute::DontDelete,
        ]
        .iter()
        .filter(|flag| (*flag & mask).0 != 0)
        .cloned()
        .collect()
    }
}

/// V8 access control values.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum V8AccessControl {
    AllCanRead = cef_v8_accesscontrol_t::V8_ACCESS_CONTROL_ALL_CAN_READ,
    AllCanWrite = cef_v8_accesscontrol_t::V8_ACCESS_CONTROL_ALL_CAN_WRITE,
    ProhibitsOverwriting = cef_v8_accesscontrol_t::V8_ACCESS_CONTROL_PROHIBITS_OVERWRITING,
}

impl V8AccessControl {
    pub(crate) fn as_mask<'a, I: 'a + Iterator<Item = &'a Self>>(attributes: I) -> cef_v8_accesscontrol_t {
        cef_v8_propertyattribute_t(attributes.fold(0, |mask, attr| mask | attr.0))
    }
    pub(crate) fn as_vec(mask: cef_v8_accesscontrol_t) -> HashSet<Self> {
        [
            V8PropertyAttribute::AllCanRead,
            V8PropertyAttribute::AllCanWrite,
            V8PropertyAttribute::ProhibitsOverwriting,
        ]
        .iter()
        .filter(|flag| (*flag & mask).0 != 0)
        .cloned()
        .collect()
    }
}

ref_counted_ptr! {
    /// Structure representing a V8 value handle. V8 handles can only be accessed
    /// from the thread on which they are created. Valid threads for creating a V8
    /// handle include the render process main thread (TID_RENDERER) and WebWorker
    /// threads. A task runner for posting tasks on the associated thread can be
    /// retrieved via the [V8Context::get_task_runner] function.
    pub struct V8Value(*mut cef_v8value_t);
}

impl V8Value {
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

    /// Returns true if the underlying handle is valid and it can be accessed
    /// on the current thread. Do not call any other functions if this function
    /// returns false.
    pub fn is_valid(&self) -> bool {
        self.0.is_valid.map(|is_valid| {
            unsafe { is_valid(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is undefined.
    pub fn is_undefined(&self) -> bool {
        self.0.is_undefined.map(|is_undefined| {
            unsafe { is_undefined(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is null.
    pub fn is_null(&self) -> bool {
        self.0.is_null.map(|is_null| {
            unsafe { is_null(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is bool.
    pub fn is_bool(&self) -> bool {
        self.0.is_bool.map(|is_bool| {
            unsafe { is_bool(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is int.
    pub fn is_int(&self) -> bool {
        self.0.is_int.map(|is_int| {
            unsafe { is_int(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is unsigned int.
    pub fn is_uint(&self) -> bool {
        self.0.is_uint.map(|is_uint| {
            unsafe { is_uint(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is double.
    pub fn is_double(&self) -> bool {
        self.0.is_double.map(|is_double| {
            unsafe { is_double(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is Date.
    pub fn is_date(&self) -> bool {
        self.0.is_date.map(|is_date| {
            unsafe { is_date(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is string.
    pub fn is_string(&self) -> bool {
        self.0.is_string.map(|is_string| {
            unsafe { is_string(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is object.
    pub fn is_object(&self) -> bool {
        self.0.is_object.map(|is_object| {
            unsafe { is_object(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is array.
    pub fn is_array(&self) -> bool {
        self.0.is_array.map(|is_array| {
            unsafe { is_array(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is an ArrayBuffer.
    pub fn is_array_buffer(&self) -> bool {
        self.0.is_array_buffer.map(|is_array_buffer| {
            unsafe { is_array_buffer(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// True if the value type is function.
    pub fn is_function(&self) -> bool {
        self.0.is_function.map(|is_function| {
            unsafe { is_function(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// Returns true if this object is pointing to the same handle as `that`
    /// object.
    pub fn is_same(&self, that: &Self) -> bool {
        self.0.is_same.map(|is_same| {
            unsafe { is_same(self.as_ptr(), that.as_ptr()) }
        }).unwrap_or(false)
    }
    /// Return a bool value.
    pub fn get_bool_value(&self) -> Option<bool> {
        if self.is_bool() {
            self.0.get_bool_value.map(|get_bool_value| {
                unsafe { get_bool_value(self.as_ptr()) }
            })
        } else {
            None
        }
    }
    /// Return an int value.
    pub fn get_int_value(&self) -> Option<i32> {
        if self.is_int() {
            self.0.get_int_value.map(|get_int_value| {
                unsafe { get_int_value(self.as_ptr()) }
            })
        } else {
            None
        }
    }
    /// Return an unsigned int value.
    pub fn get_uint_value(&self) -> Option<u32> {
        if self.is_uint() {
            self.0.get_uint_value.map(|get_uint_value| {
                unsafe { get_uint_value(self.as_ptr()) }
            })
        } else {
            None
        }
    }
    /// Return a double value.
    pub fn get_double_value(&self) -> Option<f64> {
        if self.is_double() {
            self.0.get_double_value.map(|get_double_value| {
                unsafe { get_double_value(self.as_ptr()) }
            })
        } else {
            None
        }
    }
    /// Return a Date value.
    pub fn get_date_value(&self) -> Option<SystemTime> {
        if self.is_date() {
            self.0.get_date_value.map(|get_date_value| {
                unsafe { get_date_value(self.as_ptr()) }
            })
        } else {
            None
        }
    }
    /// Return a string value.
    pub fn get_string_value(&self) -> Option<String> {
        if self.is_string() {
            self.0.get_string_value.and_then(|get_string_value| {
                CefString::from_mut_ptr(unsafe { get_string_value(self.as_ptr()) }).into_string()
            })
        } else {
            None
        }
    }
    /// Returns true if this is a user created object.
    /// 
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn is_user_created(&self) -> bool {
        self.0.is_user_created.map(|is_user_created| {
            unsafe { is_user_created(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// Returns true if the last function call resulted in an exception. This
    /// attribute exists only in the scope of the current CEF value object.
    /// 
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn has_exception(&self) -> bool {
        self.0.has_exception.map(|has_exception| {
            unsafe { has_exception(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// Returns the exception resulting from the last function call. This attribute
    /// exists only in the scope of the current CEF value object.
    /// 
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn get_exception(&self) -> Option<V8Exception> {
        self.0.get_exception.and_then(|get_exception| {
            unsafe { V8Exception::from_ptr(get_exception(self.as_ptr())) }
        })
    }
    /// Clears the last exception and returns true on success.
    /// 
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn clear_exception(&mut self) -> bool {
        self.0.clear_exception.map(|clear_exception| {
            unsafe { clear_exception(self.as_ptr_mut()) }
        }).unwrap_or(false)
    }
    /// Returns true if this object will re-throw future exceptions. This
    /// attribute exists only in the scope of the current CEF value object.
    /// 
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn will_rethrow_exceptions(&self) -> bool {
        self.0.will_rethrow_exceptions.map(|will_rethrow_exceptions| {
            unsafe { will_rethrow_exceptions(self.as_ptr()) }
        }).unwrap_or(false)
    }
    /// Set whether this object will re-throw future exceptions. By default
    /// exceptions are not re-thrown. If a exception is re-thrown the current
    /// context should not be accessed again until after the exception has been
    /// caught and not re-thrown. Returns true on success. This attribute
    /// exists only in the scope of the current CEF value object.
    /// 
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn set_rethrow_exceptions(&mut self, rethrow: bool) -> bool {
        self.0.set_rethrow_exceptions.map(|set_rethrow_exceptions| {
            unsafe { set_rethrow_exceptions(self.as_ptr_mut(), rethrow as i32) }
        }).unwrap_or(false)
    }
    /// Returns true if the object has a value with the specified identifier.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn has_value_bykey(&self, key: &str) -> bool {
        self.0.has_value_bykey.map(|has_value_bykey| {
            unsafe { has_value_bykey(self.as_ptr(), CefString::new(key).as_ptr()) }
        }).unwrap_or(false)
    }
    /// Returns true if the object has a value with the specified identifier.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn has_value_byindex(&self, index: i32) -> bool {
        self.0.has_value_byindex.map(|has_value_byindex| {
            unsafe { has_value_byindex(self.as_ptr(), index) }
        }).unwrap_or(false)
    }
    /// Deletes the value with the specified identifier and returns true on
    /// success. Returns false if this function is called incorrectly or an
    /// exception is thrown. For read-only and don't-delete values this function
    /// will return true even though deletion failed.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn delete_value_bykey(&mut self, key: &str) -> bool {
        self.0.delete_value_bykey.map(|delete_value_bykey| {
            unsafe { delete_value_bykey(self.as_ptr_mut(), CefString::new(key).as_ptr()) }
        }).unwrap_or(false)
    }
    /// Deletes the value with the specified identifier and returns true on
    /// success. Returns false if this function is called incorrectly or an
    /// exception is thrown. For read-only and don't-delete values this function
    /// will return true even though deletion failed.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn delete_value_byindex(&mut self, index: i32) -> bool {
        self.0.delete_value_byindex.map(|delete_value_byindex| {
            unsafe { delete_value_byindex(self.as_ptr_mut(), index) }
        }).unwrap_or(false)
    }
    /// Returns the value with the specified identifier on success. Returns None if
    /// this function is called incorrectly or an exception is thrown.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn get_value_bykey(&self, key: &str) -> Option<V8Value> {
        self.0.get_value_bykey.and_then(|get_value_bykey| {
            unsafe { V8Value::from_ptr(get_value_bykey(self.as_ptr(), CefString::new(key).as_ptr())) }
        })
    }
    /// Returns the value with the specified identifier on success. Returns None if
    /// this function is called incorrectly or an exception is thrown.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn get_value_byindex(&self, index: i32) -> Option<V8Value> {
        self.0.get_value_byindex.and_then(|get_value_byindex| {
            unsafe { V8Value::from_ptr(get_value_byindex(self.as_ptr(), index)) }
        })
    }
    // Associates a value with the specified identifier and returns true on
    // success. Returns false if this function is called incorrectly or an
    // exception is thrown. For read-only values this function will return true
    // even though assignment failed.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn set_value_bykey(&mut self, key: &str, value: &V8Value, attributes: impl IntoIterator<Item = V8PropertyAttribute>) -> bool {
        let attributes = V8PropertyAttribute::as_mask(attributes);
        self.0.set_value_bykey.map(|set_value_bykey| {
            unsafe { set_value_bykey(self.as_ptr_mut(), CefString::new(key).as_ptr(), value.as_ptr(), attributes) }
        }).unwrap_or(false)
    }
    // Associates a value with the specified identifier and returns true on
    // success. Returns false if this function is called incorrectly or an
    // exception is thrown. For read-only values this function will return true
    // even though assignment failed.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn set_value_byindex(&mut self, index: i32, value: V8Value) -> bool {
        self.0.set_value_byindex.map(|set_value_byindex| {
            unsafe { set_value_byindex(self.as_ptr_mut(), index, value.as_ptr()) }
        }).unwrap_or(false)
    }
    /// Registers an identifier and returns true on success. Access to the
    /// identifier will be forwarded to the [V8Accessor] instance passed to
    /// [V8Value::new_object]. Returns false if this
    /// function is called incorrectly or an exception is thrown. For read-only
    /// values this function will return true even though assignment failed.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn set_value_byaccessor(&mut self, key: &str, settings: impl IntoIterator<Item = V8AccessControl>, attributes: impl IntoIterator<Item = V8PropertyAttribute>) -> bool {
        self.0.set_value_byaccessor.map(|set_value_byaccessor| {
            let settings = V8AccessControl::as_mask(settings);
            let attributes = V8PropertyAttribute::as_mask(attributes);
            unsafe { set_value_byaccessor(self.as_ptr_mut(), CefString::new(key).as_ptr(), settings, attributes) }
        }).unwrap_or(false)
    }
    /// Read the keys for the object's values into the specified vector. Integer-
    /// based keys will also be returned as strings.
    /// 
    /// Only available on objects. Arrays and functions are also objects.
    pub fn get_keys(&self) -> Option<Vec<String>> {
        self.0.get_keys.and_then(|get_keys| {
            let list = CefStringList::new();
            if unsafe { get_keys(self.as_ptr(), list.get()) } == 0 {
                None
            } else {
                Some(list.into_vec())
            }
        })
    }
    /// Sets the user data for this object and returns true on success. Returns
    /// false if this function is called incorrectly. This function can only be
    /// called on user created objects.
    pub fn set_user_data(&mut self, user_data: impl Any + Send) -> bool {
        self.0.set_user_data.map(|set_user_data| {
            unsafe { set_user_data(self.as_ptr_mut(), UserData::new(user_data)) }
        }).unwrap_or(false)
    }
    /// Returns the user data, if any and of the right type, assigned to this object.
    pub fn get_user_data(&self) -> Option<Arc<impl std::ops::Deref<Target = Box<dyn Any + Send>>>> {
        self.0.get_user_data.and_then(|get_user_data| {
            let ptr = unsafe { get_user_data(self.as_ptr_mut()) };
            if ptr.is_null() {
                None
            } else {
                Some(unsafe { UserData::clone_raw(ptr) })
            }
        })
    }
    /// Returns the amount of externally allocated memory registered for the
    /// object.
    pub fn get_externally_allocated_memory(&self) -> i32 {
        self.0.get_externally_allocated_memory.map(|get_externally_allocated_memory| {
            unsafe { get_externally_allocated_memory(self.as_ptr()) }
        }).unwrap_or(0)
    }
    /// Adjusts the amount of registered external memory for the object. Used to
    /// give V8 an indication of the amount of externally allocated memory that is
    /// kept alive by JavaScript objects. V8 uses this information to decide when
    /// to perform global garbage collection. Each [V8Value] tracks the amount
    /// of external memory associated with it and automatically decreases the
    /// global total by the appropriate amount on its destruction.
    /// `change_in_bytes` specifies the number of bytes to adjust by. This function
    /// returns the number of bytes associated with the object after the
    /// adjustment. This function can only be called on user created objects.
    pub fn adjust_externally_allocated_memory(&mut self, change_in_bytes: i32) -> i32 {
        self.0.adjust_externally_allocated_memory.map(|adjust_externally_allocated_memory| {
            unsafe { adjust_externally_allocated_memory(self.as_ptr_mut(), change_in_bytes) }
        }).unwrap_or(0)
    }
    /// Returns the number of elements in the array.
    /// 
    /// This function is only available on arrays.
    pub fn get_array_length(&self) -> i32 {
        self.0.get_array_length.map(|get_array_length| {
            unsafe { get_array_length(self.as_ptr()) }
        }).unwrap_or(0)
    }
    // pub fn get_array_buffer_release_callback
    /// Prevent the ArrayBuffer from using it's memory block by setting the length
    /// to zero. This operation cannot be undone.
    /// 
    /// This function is only available on ArrayBuffers.
    pub fn neuter_array_buffer(&mut self) -> bool {
        self.0.neuter_array_buffer.map(|neuter_array_buffer| {
            unsafe { neuter_array_buffer(self.as_ptr_mut()) }
        }).unwrap_or(false)
    }
    /// Returns the function name.
    /// 
    /// This function is only available on functions.
    pub fn get_function_name(&self) -> Option<String> {
        self.0.get_function_name.and_then(|get_function_name| {
            unsafe { CefString::from_mut_ptr(get_function_name(self.as_ptr())) }.into_string()
        })
    }
    /// Returns the function handler or None if not a CEF-created function.
    /// 
    /// This function is only available on functions.
    pub fn get_function_handler(&self) -> Option<impl Fn(&str, &mut V8Value, &[&mut V8Value]) -> Result<V8Value, String> + 'static> {
        self.0.get_function_handler.and_then(|get_function_handler| {
            let handler = unsafe { get_function_handler(self.as_ptr()) };
            if handler.is_null() {
                None
            } else {
                let this = unsafe { RefCounted::<cef_v8handler_t>::make_temp(handler) };
                Some(*this)
            }
        })
    }
    /// Execute the function using the current V8 context. This function should
    /// only be called from within the scope of a [V8Handler] or
    /// [V8Accessor] callback, or in combination with calling [V8Context::enter] and
    /// [V8Context::exit] on a stored [V8Context] reference. `object` is the receiver
    /// ('this' object) of the function. If `object` is None the current context's
    /// global object will be used. `arguments` is the list of arguments that will
    /// be passed to the function. Returns the function return value on success.
    /// Returns None if this function is called incorrectly or an exception is
    /// thrown.
    /// 
    /// This function is only available on functions.
    pub fn execute_function(&mut self, object: Option<&mut V8Value>, arguments: &[&mut V8Value]) -> Option<V8Value> {
        self.0.execute_function.and_then(|execute_function| {
            let count = arguments.len();
            let result = unsafe { execute_function(self.as_ptr(), object.map(|obj| obj.as_ptr_mut()).unwrap_or_else(null_mut), count, arguments.as_mut_ptr()) };
            V8Value::from_ptr(result)
        })
    }
    /// Execute the function using the specified V8 context. `object` is the
    /// receiver ('this' object) of the function. If `object` is None the specified
    /// context's global object will be used. `arguments` is the list of arguments
    /// that will be passed to the function. Returns the function return value on
    /// success. Returns None if this function is called incorrectly or an
    /// exception is thrown.
    /// 
    /// This function is only available on functions.
    pub fn execute_function_with_context(&mut self, context: &mut V8Context, object: Option<&mut V8Value>, arguments: &[&mut V8Value]) -> Option<V8Value> {
        self.0.execute_function.and_then(|execute_function| {
            let count = arguments.len();
            let result = unsafe { execute_function(self.as_ptr(), context.as_ptr_mut(), object.map(|obj| obj.as_ptr_mut()).unwrap_or_else(null_mut), count, arguments.as_mut_ptr()) };
            V8Value::from_ptr(result)
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

struct V8ArrayBufferReleaseCallbackWrapper(<cef_v8array_buffer_release_callback_t as RefCounterWrapped>::Wrapper);

impl V8ArrayBufferReleaseCallbackWrapper {
    fn new(
        delegate: impl FnOnce(*mut u8) + 'static
    ) -> *mut cef_v8array_buffer_release_callback_t {
        let rc = RefCounted::new(
            cef_v8array_buffer_release_callback_t {
                base: unsafe { std::mem::zeroed() },
                release_buffer: Some(Self::release_buffer),
            },
            Some(Box::new(delegate)),
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

struct V8HandlerWrapper(<cef_v8handler_t as RefCounterWrapped>::Wrapper);

impl V8HandlerWrapper {
    fn new(
        delegate: impl Fn(&str, &mut V8Value, &[&mut V8Value]) -> Result<V8Value, String> + 'static
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

/// User Data wrapper used for storing in V8Value objects that takes care of
/// memory management between CEF and Rust.
#[repr(C)]
pub(crate) struct UserData {
    base: cef_base_ref_counted_t,
    pub(crate) user_data: Box<dyn Any + Send>,
}

impl UserData {
    pub(crate) fn new(user_data: impl Any + Send) -> *mut cef_base_ref_counted_t {
        let result = Arc::new(Self {
            base: cef_base_ref_counted_t {
                size: std::mem::size_of::<Self>(),
                add_ref: Some(Self::add_ref),
                release: Some(Self::release),
                has_one_ref: Some(Self::has_one_ref),
            },
            user_data: Box::new(user_data),
        });
        result.into_raw() as *mut _
    }
    pub(crate) unsafe fn clone_raw(ptr: *mut cef_base_ref_counted_t) -> Arc<Self> {
        let this = Arc::from_raw(ptr as *const UserData);
        this.clone().into_raw();
        this
    }

    extern "C" fn add_ref(self_: *mut cef_base_ref_counted_t) {
        let this = unsafe { Arc::from_raw(self_ as *const UserData) };
        this.clone().into_raw();
        this.into_raw();
    }
    extern "C" fn release(self_: *mut cef_base_ref_counted_t) -> i32 {
        let this = unsafe { Arc::from_raw(self_ as *const UserData) };
        (this.strong_count() <= 1) as i32
    }
    extern "C" fn has_one_ref(self_: *mut cef_base_ref_counted_t) -> i32 {
        let this = unsafe { Arc::from_raw(self_ as *const UserData) };
        let result = this.strong_count() == 1;
        this.into_raw();
        result as i32
    }
    extern "C" fn has_at_least_one_ref(self_: *mut cef_base_ref_counted_t) -> i32 {
        let this = unsafe { Arc::from_raw(self_ as *const UserData) };
        let result = this.strong_count() >= 1;
        this.into_raw();
        result as i32
    }
}

impl std::ops::Deref for UserData {
    fn deref(&self) -> &Box<dyn Any + Send> {
        &self.user_data
    }
}
