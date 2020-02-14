use cef_sys::{
    cef_base_ref_counted_t, cef_register_extension, cef_string_t, cef_time_from_doublet,
    cef_time_to_doublet, cef_v8_accesscontrol_t, cef_v8_propertyattribute_t, cef_v8accessor_t,
    cef_v8array_buffer_release_callback_t, cef_v8context_get_current_context,
    cef_v8context_get_entered_context, cef_v8context_in_context, cef_v8context_t,
    cef_v8exception_t, cef_v8handler_t, cef_v8interceptor_t, cef_v8stack_frame_t,
    cef_v8stack_trace_get_current, cef_v8stack_trace_t, cef_v8value_create_array,
    cef_v8value_create_array_buffer, cef_v8value_create_bool, cef_v8value_create_date,
    cef_v8value_create_double, cef_v8value_create_function, cef_v8value_create_int,
    cef_v8value_create_null, cef_v8value_create_object, cef_v8value_create_string,
    cef_v8value_create_uint, cef_v8value_create_undefined, cef_v8value_t,
};
use parking_lot::Mutex;
use std::{
    any::Any,
    cell::RefCell,
    collections::HashSet,
    convert::TryFrom,
    ptr::null_mut,
    time::{Duration, SystemTime, SystemTimeError},
};

use crate::{
    browser::Browser,
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
    string::{CefString, CefStringList},
    task::TaskRunner,
};

ref_counted_ptr! {
    /// Structure representing a V8 context handle. V8 handles can only be accessed
    /// from the thread on which they are created. Valid threads for creating a V8
    /// handle include the render process main thread (TID_RENDERER) and WebWorker
    /// threads. A task runner for posting tasks on the associated thread can be
    /// retrieved via the [V8Context::get_task_runner] function.
    pub struct V8Context(*mut cef_v8context_t);
}

impl V8Context {
    /// Returns the current (top) context object in the V8 context stack.
    pub fn get_current() -> Option<Self> {
        unsafe { Self::from_ptr(cef_v8context_get_current_context()) }
    }
    /// Returns the entered (bottom) context object in the V8 context stack.
    pub fn get_entered() -> Option<Self> {
        unsafe { Self::from_ptr(cef_v8context_get_entered_context()) }
    }
    /// Returns true if V8 is currently inside a context.
    pub fn in_context() -> bool {
        unsafe { cef_v8context_in_context() != 0 }
    }
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
    pub fn register_extension(
        extension_name: &str,
        javascript_code: &str,
        handler: impl Fn(&str, V8Value, &[V8Value]) -> Result<V8Value, String> + Sync + Send + 'static,
    ) {
        let name = CefString::new(extension_name);
        let js = CefString::new(javascript_code);
        unsafe {
            cef_register_extension(
                name.as_ptr(),
                js.as_ptr(),
                V8HandlerWrapper::new(Box::new(handler)).wrap().into_raw(),
            );
        }
    }
    /// Returns the task runner associated with this context. V8 handles can only
    /// be accessed from the thread on which they are created. This function can be
    /// called on any render process thread.
    pub fn get_task_runner(&self) -> TaskRunner {
        let get_task_runner = self.0.get_task_runner.unwrap();
        unsafe { TaskRunner::from_ptr_unchecked(get_task_runner(self.as_ptr())) }
    }
    /// Returns true if the underlying handle is valid and it can be accessed
    /// on the current thread. Do not call any other functions if this function
    /// returns false.
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the browser for this context. This function will return None
    /// for WebWorker contexts.
    pub fn get_browser(&self) -> Option<Browser> {
        self.0
            .get_browser
            .and_then(|get_browser| unsafe { Browser::from_ptr(get_browser(self.as_ptr())) })
    }
    /// Returns the frame for this context. This function will return None
    /// for WebWorker contexts.
    pub fn get_frame(&self) -> Option<Frame> {
        self.0
            .get_frame
            .and_then(|get_frame| unsafe { Frame::from_ptr(get_frame(self.as_ptr())) })
    }
    /// Returns the global object for this context or None if scripting is disabled.
    /// The context must be entered before calling this function.
    pub fn get_global(&self) -> Option<V8Value> {
        self.0
            .get_global
            .and_then(|get_global| unsafe { V8Value::from_ptr(get_global(self.as_ptr())) })
    }
    /// Enter this context. A context must be explicitly entered before creating a
    /// V8 Object, Array, Function or Date asynchronously. [exit] must be called
    /// the same number of times as [enter] before releasing this context. V8
    /// objects belong to the context in which they are created. Returns true
    /// if the scope was entered successfully.
    pub fn enter(&self) -> bool {
        self.0
            .enter
            .map(|enter| unsafe { enter(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Exit this context. Call this function only after calling [enter]. Returns
    /// true if the scope was exited successfully.
    pub fn exit(&self) -> bool {
        self.0
            .exit
            .map(|exit| unsafe { exit(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Convenience function to wrap a closure in an [enter] and [exit] call.
    /// If enter fails, the closure is not executed and None is returned. If exit fails, None is returned.
    pub fn execute_in_context<T>(&self, fun: impl FnOnce() -> T) -> Option<T> {
        if self.enter() {
            let result = fun();
            if self.exit() {
                return Some(result);
            }
        }
        None
    }
    /// Returns true if this object is pointing to the same handle as `that`
    /// object.
    pub fn is_same(&self, that: &Self) -> bool {
        self.0
            .is_same
            .map(|is_same| unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Execute a string of JavaScript code in this V8 context. The `script_url`
    /// parameter is the URL where the script in question can be found, if any. The
    /// `start_line` parameter is the base line number to use for error reporting.
    pub fn eval(
        &self,
        code: &str,
        script_url: &str,
        start_line: i32,
    ) -> Result<V8Value, V8Exception> {
        let eval = self.0.eval.unwrap();
        let mut retval = null_mut();
        let mut exception = null_mut();
        if unsafe {
            eval(
                self.as_ptr(),
                CefString::new(code).as_ptr(),
                CefString::new(script_url).as_ptr(),
                start_line,
                &mut retval,
                &mut exception,
            ) == 1
        } {
            Ok(unsafe { V8Value::from_ptr_unchecked(retval) })
        } else {
            Err(unsafe { V8Exception::from_ptr_unchecked(exception) })
        }
    }

    /// Returns the stack trace for the currently active context. `frame_limit` is
    /// the maximum number of frames that will be captured.
    pub fn get_current_stacktrace(frame_limit: i32) -> Vec<V8StackFrame> {
        let frames =
            unsafe { V8StackTrace::from_ptr_unchecked(cef_v8stack_trace_get_current(frame_limit)) };
        frames.into()
    }
}

ref_counted_ptr! {
    /// Structure representing a V8 exception. The functions of this structure may be
    /// called on any render process thread.
    pub struct V8Exception(*mut cef_v8exception_t);
}

impl V8Exception {
    /// Returns the exception message.
    pub fn get_message(&self) -> String {
        self.0
            .get_message
            .and_then(|get_message| {
                unsafe { CefString::from_mut_ptr(get_message(self.as_ptr())) }
                    .map(|s| <String as From<&CefString>>::from(s))
            })
            .unwrap_or_default()
    }
    /// Returns the line of source code that the exception occurred within.
    pub fn get_source_line(&self) -> String {
        self.0
            .get_source_line
            .and_then(|get_source_line| {
                unsafe { CefString::from_mut_ptr(get_source_line(self.as_ptr())) }
                    .map(|s| <String as From<&CefString>>::from(s))
            })
            .unwrap_or_default()
    }
    /// Returns the resource name for the script from where the function causing
    /// the error originates.
    pub fn get_script_resource_name(&self) -> String {
        self.0
            .get_script_resource_name
            .and_then(|get_script_resource_name| {
                unsafe { CefString::from_mut_ptr(get_script_resource_name(self.as_ptr())) }
                    .map(|s| <String as From<&CefString>>::from(s))
            })
            .unwrap_or_default()
    }
    /// Returns the 1-based number of the line where the error occurred or 0 if the
    /// line number is unknown.
    pub fn get_line_number(&self) -> i32 {
        self.0
            .get_line_number
            .map(|get_line_number| unsafe { get_line_number(self.as_ptr()) })
            .unwrap_or_default()
    }
    /// Returns the index within the script of the first character where the error
    /// occurred.
    pub fn get_start_position(&self) -> i32 {
        self.0
            .get_start_position
            .map(|get_start_position| unsafe { get_start_position(self.as_ptr()) })
            .unwrap_or_default()
    }
    /// Returns the index within the script of the last character where the error
    /// occurred.
    pub fn get_end_position(&self) -> i32 {
        self.0
            .get_end_position
            .map(|get_end_position| unsafe { get_end_position(self.as_ptr()) })
            .unwrap_or_default()
    }
    /// Returns the index within the line of the first character where the error
    /// occurred.
    pub fn get_start_column(&self) -> i32 {
        self.0
            .get_start_column
            .map(|get_start_column| unsafe { get_start_column(self.as_ptr()) })
            .unwrap_or_default()
    }
    /// Returns the index within the line of the last character where the error
    /// occurred.
    pub fn get_end_column(&self) -> i32 {
        self.0
            .get_end_column
            .map(|get_end_column| unsafe { get_end_column(self.as_ptr()) })
            .unwrap_or_default()
    }
}

ref_counted_ptr! {
    /// Structure representing a V8 stack frame handle. V8 handles can only be
    /// accessed from the thread on which they are created. Valid threads for
    /// creating a V8 handle include the render process main thread (TID_RENDERER)
    /// and WebWorker threads. A task runner for posting tasks on the associated
    /// thread can be retrieved via the [V8Context::get_task_runner] function.
    pub struct V8StackFrame(*mut cef_v8stack_frame_t);
}

impl V8StackFrame {
    /// Returns true if the underlying handle is valid and it can be accessed
    /// on the current thread. Do not call any other functions if this function
    /// returns false.
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the name of the resource script that contains the function.
    pub fn get_script_name(&self) -> String {
        self.0
            .get_script_name
            .and_then(|get_script_name| {
                unsafe { CefString::from_mut_ptr(get_script_name(self.as_ptr())) }
                    .map(|s| <String as From<&CefString>>::from(s))
            })
            .unwrap_or_default()
    }
    /// Returns the name of the resource script that contains the function or the
    /// sourceURL value if the script name is undefined and its source ends with a
    /// `"//@ sourceURL=..."` string.
    pub fn get_script_name_or_source_url(&self) -> String {
        self.0
            .get_script_name_or_source_url
            .and_then(|get_script_name_or_source_url| {
                unsafe { CefString::from_mut_ptr(get_script_name_or_source_url(self.as_ptr())) }
                    .map(|s| <String as From<&CefString>>::from(s))
            })
            .unwrap_or_default()
    }
    /// Returns the name of the function.
    pub fn get_function_name(&self) -> String {
        self.0
            .get_function_name
            .and_then(|get_function_name| {
                unsafe { CefString::from_mut_ptr(get_function_name(self.as_ptr())) }
                    .map(|s| <String as From<&CefString>>::from(s))
            })
            .unwrap_or_default()
    }
    /// Returns the 1-based line number for the function call or 0 if unknown.
    pub fn get_line_number(&self) -> i32 {
        self.0
            .get_line_number
            .map(|get_line_number| unsafe { get_line_number(self.as_ptr()) })
            .unwrap_or_default()
    }
    /// Returns the 1-based column offset on the line for the function call or 0 if
    /// unknown.
    pub fn get_column(&self) -> i32 {
        self.0
            .get_column
            .map(|get_column| unsafe { get_column(self.as_ptr()) })
            .unwrap_or_default()
    }
    /// Returns true if the function was compiled using eval().
    pub fn is_eval(&self) -> bool {
        self.0
            .is_eval
            .map(|is_eval| unsafe { is_eval(self.as_ptr()) != 0 })
            .unwrap_or_default()
    }
    /// Returns true if the function was called as a constructor via "new".
    pub fn is_constructor(&self) -> bool {
        self.0
            .is_constructor
            .map(|is_constructor| unsafe { is_constructor(self.as_ptr()) != 0 })
            .unwrap_or_default()
    }
}

ref_counted_ptr! {
    pub(crate) struct V8StackTrace(*mut cef_v8stack_trace_t);
}

impl From<V8StackTrace> for Vec<V8StackFrame> {
    fn from(trace: V8StackTrace) -> Vec<V8StackFrame> {
        let count = trace
            .0
            .get_frame_count
            .map(|get_frame_count| unsafe { get_frame_count(trace.0.as_ptr()) })
            .unwrap_or(0);
        if let Some(get_frame) = trace.0.get_frame {
            (0..count)
                .map(|idx| unsafe {
                    V8StackFrame::from_ptr_unchecked(get_frame(trace.0.as_ptr(), idx))
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// V8 property attribute values.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum V8PropertyAttribute {
    /// Not writeable
    ReadOnly = cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_READONLY.0 as isize,
    /// Not enumerable
    DontEnum = cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_DONTENUM.0 as isize,
    /// Not configurable
    DontDelete = cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_DONTDELETE.0 as isize,
}

impl V8PropertyAttribute {
    pub(crate) fn as_mask<'a, I: 'a + Iterator<Item = &'a Self>>(
        attributes: I,
    ) -> cef_v8_propertyattribute_t {
        cef_v8_propertyattribute_t(attributes.fold(0, |mask, attr| mask | (*attr as crate::CEnumType)))
    }
    pub(crate) fn as_vec(mask: cef_v8_propertyattribute_t) -> HashSet<Self> {
        [
            V8PropertyAttribute::ReadOnly,
            V8PropertyAttribute::DontEnum,
            V8PropertyAttribute::DontDelete,
        ]
        .iter()
        .filter(|flag| (**flag as crate::CEnumType) & mask.0 != 0)
        .cloned()
        .collect()
    }
}

/// V8 access control values.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum V8AccessControl {
    AllCanRead = cef_v8_accesscontrol_t::V8_ACCESS_CONTROL_ALL_CAN_READ.0 as isize,
    AllCanWrite = cef_v8_accesscontrol_t::V8_ACCESS_CONTROL_ALL_CAN_WRITE.0 as isize,
    ProhibitsOverwriting = cef_v8_accesscontrol_t::V8_ACCESS_CONTROL_PROHIBITS_OVERWRITING.0 as isize,
}

impl V8AccessControl {
    pub(crate) fn as_mask<'a, I: 'a + Iterator<Item = &'a Self>>(
        attributes: I,
    ) -> cef_v8_accesscontrol_t {
        cef_v8_accesscontrol_t(attributes.fold(0, |mask, attr| mask | (*attr as crate::CEnumType)))
    }
    pub(crate) fn as_vec(mask: cef_v8_accesscontrol_t) -> HashSet<Self> {
        [
            V8AccessControl::AllCanRead,
            V8AccessControl::AllCanWrite,
            V8AccessControl::ProhibitsOverwriting,
        ]
        .iter()
        .filter(|flag| (**flag as crate::CEnumType) & mask.0 != 0)
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
        unsafe { V8Value::from_ptr_unchecked(cef_v8value_create_undefined()) }
    }
    /// Create a new V8Value object of type null.
    pub fn null() -> Self {
        unsafe { V8Value::from_ptr_unchecked(cef_v8value_create_null()) }
    }
    /// Create a new V8Value object of type object with optional accessor
    /// and/or interceptor. This function should only be called from within the scope
    /// of a [RenderProcessHandlerCallbacks], [V8Handler] or [V8AccessorCallbacks]
    /// callback, or in combination with calling [V8Context::enter] and [V8Context::exit] on a stored
    /// [V8Context] reference.
    pub fn new_object(
        accessor: Option<V8Accessor>,
        interceptor: Option<V8Interceptor>,
    ) -> Self {
        unsafe {
            V8Value::from_ptr_unchecked(cef_v8value_create_object(
                accessor
                    .map(V8Accessor::into_raw)
                    .unwrap_or_else(null_mut),
                interceptor
                    .map(V8Interceptor::into_raw)
                    .unwrap_or_else(null_mut),
            ))
        }
    }
    /// Create a new V8Value object of type array with the specified `length`.
    /// If `length` is negative the returned array will have length 0. This function
    /// should only be called from within the scope of a
    /// [RenderProcessHandlerCallbacks], [V8Handler] or [V8AccessorCallbacks] callback,
    /// or in combination with calling [V8Context::enter] and [V8Context::exit] on a stored V8Context
    /// reference.
    pub fn new_array(length: i32) -> Self {
        unsafe { V8Value::from_ptr_unchecked(cef_v8value_create_array(length)) }
    }
    /// Create a new V8Value object of type ArrayBuffer which wraps the
    /// provided `buffer` (without copying it). This function should only
    /// be called from within the scope of a [RenderProcessHandlerCallbacks], [V8Handler]
    /// or [V8AccessorCallbacks] callback, or in combination with calling
    /// [V8Context::enter] and [V8Context::exit] on a stored [V8Context]
    /// reference.
    pub fn new_array_buffer(mut buffer: Vec<u8>) -> Self {
        let length = buffer.len();
        let capacity = buffer.capacity();
        let ptr = buffer.as_mut_ptr();
        std::mem::forget(buffer);
        unsafe {
            V8Value::from_ptr_unchecked(cef_v8value_create_array_buffer(
                ptr as *mut _,
                length,
                V8ArrayBufferReleaseCallbackWrapper::new(move |ptr| {
                    Vec::from_raw_parts(ptr, length, capacity);
                })
                .wrap()
                .into_raw(),
            ))
        }
    }
    /// Create a new V8Value object of type function. This function
    /// should only be called from within the scope of a
    /// [RenderProcessHandlerCallbacks], [V8Handler] or [V8AccessorCallbacks] callback,
    /// or in combination with calling [V8Context::enter] and [V8Context::exit] on a stored [V8Context]
    /// reference.
    pub fn new_function(
        name: &str,
        handler: impl Fn(&str, V8Value, &[V8Value]) -> Result<V8Value, String> + Sync + Send + 'static,
    ) -> Self {
        let name = CefString::new(name);
        unsafe {
            V8Value::from_ptr_unchecked(cef_v8value_create_function(
                name.as_ptr(),
                V8HandlerWrapper::new(Box::new(handler)).wrap().into_raw(),
            ))
        }
    }

    /// Returns true if the underlying handle is valid and it can be accessed
    /// on the current thread. Do not call any other functions if this function
    /// returns false.
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is undefined.
    pub fn is_undefined(&self) -> bool {
        self.0
            .is_undefined
            .map(|is_undefined| unsafe { is_undefined(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is null.
    pub fn is_null(&self) -> bool {
        self.0
            .is_null
            .map(|is_null| unsafe { is_null(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is bool.
    pub fn is_bool(&self) -> bool {
        self.0
            .is_bool
            .map(|is_bool| unsafe { is_bool(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is int.
    pub fn is_int(&self) -> bool {
        self.0
            .is_int
            .map(|is_int| unsafe { is_int(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is unsigned int.
    pub fn is_uint(&self) -> bool {
        self.0
            .is_uint
            .map(|is_uint| unsafe { is_uint(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is double.
    pub fn is_double(&self) -> bool {
        self.0
            .is_double
            .map(|is_double| unsafe { is_double(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is Date.
    pub fn is_date(&self) -> bool {
        self.0
            .is_date
            .map(|is_date| unsafe { is_date(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is string.
    pub fn is_string(&self) -> bool {
        self.0
            .is_string
            .map(|is_string| unsafe { is_string(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is object.
    pub fn is_object(&self) -> bool {
        self.0
            .is_object
            .map(|is_object| unsafe { is_object(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is array.
    pub fn is_array(&self) -> bool {
        self.0
            .is_array
            .map(|is_array| unsafe { is_array(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is an ArrayBuffer.
    pub fn is_array_buffer(&self) -> bool {
        self.0
            .is_array_buffer
            .map(|is_array_buffer| unsafe { is_array_buffer(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// True if the value type is function.
    pub fn is_function(&self) -> bool {
        self.0
            .is_function
            .map(|is_function| unsafe { is_function(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if this object is pointing to the same handle as `that`
    /// object.
    pub fn is_same(&self, that: &Self) -> bool {
        self.0
            .is_same
            .map(|is_same| unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Return a bool value.
    pub fn get_bool_value(&self) -> Option<bool> {
        if self.is_bool() {
            self.0
                .get_bool_value
                .map(|get_bool_value| unsafe { get_bool_value(self.as_ptr()) != 0 })
        } else {
            None
        }
    }
    /// Return an int value.
    pub fn get_int_value(&self) -> Option<i32> {
        if self.is_int() {
            self.0
                .get_int_value
                .map(|get_int_value| unsafe { get_int_value(self.as_ptr()) })
        } else {
            None
        }
    }
    /// Return an unsigned int value.
    pub fn get_uint_value(&self) -> Option<u32> {
        if self.is_uint() {
            self.0
                .get_uint_value
                .map(|get_uint_value| unsafe { get_uint_value(self.as_ptr()) })
        } else {
            None
        }
    }
    /// Return a double value.
    pub fn get_double_value(&self) -> Option<f64> {
        if self.is_double() {
            self.0
                .get_double_value
                .map(|get_double_value| unsafe { get_double_value(self.as_ptr()) })
        } else {
            None
        }
    }
    /// Return a Date value.
    pub fn get_date_value(&self) -> Option<SystemTime> {
        if self.is_date() {
            self.0.get_date_value.map(|get_date_value| {
                let value = unsafe { get_date_value(self.as_ptr()) };
                let mut fvalue = 0.0;
                unsafe {
                    cef_time_to_doublet(&value, &mut fvalue);
                }
                SystemTime::UNIX_EPOCH + Duration::from_secs_f64(fvalue)
            })
        } else {
            None
        }
    }
    /// Return a string value.
    pub fn get_string_value(&self) -> Option<String> {
        if self.is_string() {
            self.0.get_string_value.and_then(|get_string_value| {
                unsafe { CefString::from_mut_ptr(get_string_value(self.as_ptr())) }
                    .map(|s| <&CefString as Into<_>>::into(s))
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
        self.0
            .is_user_created
            .map(|is_user_created| unsafe { is_user_created(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if the last function call resulted in an exception. This
    /// attribute exists only in the scope of the current CEF value object.
    ///
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn has_exception(&self) -> bool {
        self.0
            .has_exception
            .map(|has_exception| unsafe { has_exception(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the exception resulting from the last function call. This attribute
    /// exists only in the scope of the current CEF value object.
    ///
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn get_exception(&self) -> Option<V8Exception> {
        self.0.get_exception.and_then(|get_exception| unsafe {
            V8Exception::from_ptr(get_exception(self.as_ptr()))
        })
    }
    /// Clears the last exception and returns true on success.
    ///
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn clear_exception(&self) -> bool {
        self.0
            .clear_exception
            .map(|clear_exception| unsafe { clear_exception(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns true if this object will re-throw future exceptions. This
    /// attribute exists only in the scope of the current CEF value object.
    ///
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn will_rethrow_exceptions(&self) -> bool {
        self.0
            .will_rethrow_exceptions
            .map(|will_rethrow_exceptions| unsafe { will_rethrow_exceptions(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Set whether this object will re-throw future exceptions. By default
    /// exceptions are not re-thrown. If a exception is re-thrown the current
    /// context should not be accessed again until after the exception has been
    /// caught and not re-thrown. Returns true on success. This attribute
    /// exists only in the scope of the current CEF value object.
    ///
    /// Only available on objects. Arrays and
    /// functions are also objects.
    pub fn set_rethrow_exceptions(&self, rethrow: bool) -> bool {
        self.0
            .set_rethrow_exceptions
            .map(|set_rethrow_exceptions| unsafe {
                set_rethrow_exceptions(self.as_ptr(), rethrow as i32) != 0
            })
            .unwrap_or(false)
    }
    /// Returns true if the object has a value with the specified identifier.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn has_value_bykey(&self, key: &str) -> bool {
        self.0
            .has_value_bykey
            .map(|has_value_bykey| unsafe {
                has_value_bykey(self.as_ptr(), CefString::new(key).as_ptr()) != 0
            })
            .unwrap_or(false)
    }
    /// Returns true if the object has a value with the specified identifier.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn has_value_byindex(&self, index: i32) -> bool {
        self.0
            .has_value_byindex
            .map(|has_value_byindex| unsafe { has_value_byindex(self.as_ptr(), index) != 0 })
            .unwrap_or(false)
    }
    /// Deletes the value with the specified identifier and returns true on
    /// success. Returns false if this function is called incorrectly or an
    /// exception is thrown. For read-only and don't-delete values this function
    /// will return true even though deletion failed.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn delete_value_bykey(&self, key: &str) -> bool {
        self.0
            .delete_value_bykey
            .map(|delete_value_bykey| unsafe {
                delete_value_bykey(self.as_ptr(), CefString::new(key).as_ptr()) != 0
            })
            .unwrap_or(false)
    }
    /// Deletes the value with the specified identifier and returns true on
    /// success. Returns false if this function is called incorrectly or an
    /// exception is thrown. For read-only and don't-delete values this function
    /// will return true even though deletion failed.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn delete_value_byindex(&self, index: i32) -> bool {
        self.0
            .delete_value_byindex
            .map(|delete_value_byindex| unsafe { delete_value_byindex(self.as_ptr(), index) != 0 })
            .unwrap_or(false)
    }
    /// Returns the value with the specified identifier on success. Returns None if
    /// this function is called incorrectly or an exception is thrown.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn get_value_bykey(&self, key: &str) -> Option<V8Value> {
        self.0.get_value_bykey.and_then(|get_value_bykey| unsafe {
            V8Value::from_ptr(get_value_bykey(self.as_ptr(), CefString::new(key).as_ptr()))
        })
    }
    /// Returns the value with the specified identifier on success. Returns None if
    /// this function is called incorrectly or an exception is thrown.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn get_value_byindex(&self, index: i32) -> Option<V8Value> {
        self.0
            .get_value_byindex
            .and_then(|get_value_byindex| unsafe {
                V8Value::from_ptr(get_value_byindex(self.as_ptr(), index))
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
    pub fn set_value_bykey(
        &self,
        key: &str,
        value: V8Value,
        attributes: &[V8PropertyAttribute],
    ) -> bool {
        let attributes = V8PropertyAttribute::as_mask(attributes.iter());
        self.0
            .set_value_bykey
            .map(|set_value_bykey| unsafe {
                set_value_bykey(
                    self.as_ptr(),
                    CefString::new(key).as_ptr(),
                    value.into_raw(),
                    attributes,
                ) != 0
            })
            .unwrap_or(false)
    }
    // Associates a value with the specified identifier and returns true on
    // success. Returns false if this function is called incorrectly or an
    // exception is thrown. For read-only values this function will return true
    // even though assignment failed.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn set_value_byindex(&self, index: i32, value: V8Value) -> bool {
        self.0
            .set_value_byindex
            .map(|set_value_byindex| unsafe {
                set_value_byindex(self.as_ptr(), index, value.as_ptr()) != 0
            })
            .unwrap_or(false)
    }
    /// Registers an identifier and returns true on success. Access to the
    /// identifier will be forwarded to the [V8AccessorCallbacks] instance passed to
    /// [V8Value::new_object]. Returns false if this
    /// function is called incorrectly or an exception is thrown. For read-only
    /// values this function will return true even though assignment failed.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    /// String- and integer-based keys can be used interchangably with the
    /// framework converting between them as necessary.
    pub fn set_value_byaccessor(
        &self,
        key: &str,
        settings: &[V8AccessControl],
        attributes: &[V8PropertyAttribute],
    ) -> bool {
        self.0
            .set_value_byaccessor
            .map(|set_value_byaccessor| {
                let settings = V8AccessControl::as_mask(settings.iter());
                let attributes = V8PropertyAttribute::as_mask(attributes.iter());
                unsafe {
                    set_value_byaccessor(
                        self.as_ptr(),
                        CefString::new(key).as_ptr(),
                        settings,
                        attributes,
                    ) != 0
                }
            })
            .unwrap_or(false)
    }
    /// Read the keys for the object's values into the specified vector. Integer-
    /// based keys will also be returned as strings.
    ///
    /// Only available on objects. Arrays and functions are also objects.
    pub fn get_keys(&self) -> Option<Vec<String>> {
        self.0.get_keys.and_then(|get_keys| {
            let mut list = CefStringList::new();
            if unsafe { get_keys(self.as_ptr(), list.as_mut_ptr()) } == 0 {
                None
            } else {
                Some(
                    list.into_iter()
                        .map(|s| <CefString as Into<_>>::into(s))
                        .collect(),
                )
            }
        })
    }
    /// Sets the user data for this object and returns true on success. Returns
    /// false if this function is called incorrectly. This function can only be
    /// called on user created objects.
    pub fn set_user_data(&self, user_data: impl Any + Sync + Send) -> bool {
        self.0
            .set_user_data
            .map(|set_user_data| unsafe {
                set_user_data(self.as_ptr(), UserData::new(user_data).into_raw() as _) != 0
            })
            .unwrap_or(false)
    }
    /// Returns the user data, if any and of the right type, assigned to this object.
    pub fn get_user_data(&self) -> Option<UserData> {
        self.0.get_user_data.and_then(|get_user_data| {
            let ptr = unsafe { get_user_data(self.as_ptr()) };
            unsafe { UserData::from_ptr(ptr as _) }
        })
    }
    /// Returns the amount of externally allocated memory registered for the
    /// object.
    pub fn get_externally_allocated_memory(&self) -> i32 {
        self.0
            .get_externally_allocated_memory
            .map(|get_externally_allocated_memory| unsafe {
                get_externally_allocated_memory(self.as_ptr())
            })
            .unwrap_or(0)
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
    pub fn adjust_externally_allocated_memory(&self, change_in_bytes: i32) -> i32 {
        self.0
            .adjust_externally_allocated_memory
            .map(|adjust_externally_allocated_memory| unsafe {
                adjust_externally_allocated_memory(self.as_ptr(), change_in_bytes)
            })
            .unwrap_or(0)
    }
    /// Returns the number of elements in the array.
    ///
    /// This function is only available on arrays.
    pub fn get_array_length(&self) -> i32 {
        self.0
            .get_array_length
            .map(|get_array_length| unsafe { get_array_length(self.as_ptr()) })
            .unwrap_or(0)
    }
    // pub fn get_array_buffer_release_callback
    /// Prevent the ArrayBuffer from using it's memory block by setting the length
    /// to zero. This operation cannot be undone.
    ///
    /// This function is only available on ArrayBuffers.
    pub fn neuter_array_buffer(&self) -> bool {
        self.0
            .neuter_array_buffer
            .map(|neuter_array_buffer| unsafe { neuter_array_buffer(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the function name.
    ///
    /// This function is only available on functions.
    pub fn get_function_name(&self) -> Option<String> {
        self.0.get_function_name.and_then(|get_function_name| {
            unsafe { CefString::from_mut_ptr(get_function_name(self.as_ptr())) }
                .map(|s| <&CefString as Into<_>>::into(s))
        })
    }
    /// Returns the function handler or None if not a CEF-created function.
    ///
    /// This function is only available on functions.
    pub fn get_function_handler(&self) -> Option<V8Handler> {
        self.0
            .get_function_handler
            .map(|get_function_handler| unsafe {
                let handler = get_function_handler(self.as_ptr());
                V8Handler::from_ptr_unchecked(handler)
            })
    }
    /// Execute the function using the current V8 context. This function should
    /// only be called from within the scope of a [V8Handler] or
    /// [V8AccessorCallbacks] callback, or in combination with calling [V8Context::enter] and
    /// [V8Context::exit] on a stored [V8Context] reference. `object` is the receiver
    /// ('this' object) of the function. If `object` is None the current context's
    /// global object will be used. `arguments` is the list of arguments that will
    /// be passed to the function. Returns the function return value on success.
    /// Returns None if this function is called incorrectly or an exception is
    /// thrown.
    ///
    /// This function is only available on functions.
    pub fn execute_function(
        &self,
        object: Option<V8Value>,
        arguments: &[V8Value],
    ) -> Option<V8Value> {
        self.0.execute_function.and_then(|execute_function| {
            let count = arguments.len();
            unsafe {
                let result = execute_function(
                    self.as_ptr(),
                    object.map(|obj| obj.into_raw()).unwrap_or_else(null_mut),
                    count,
                    arguments
                        .iter()
                        .map(|v| v.clone().into_raw())
                        .collect::<Vec<_>>()
                        .as_ptr(),
                );
                V8Value::from_ptr(result)
            }
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
    pub fn execute_function_with_context(
        &self,
        context: V8Context,
        object: Option<V8Value>,
        arguments: &[V8Value],
    ) -> Option<V8Value> {
        self.0
            .execute_function_with_context
            .and_then(|execute_function_with_context| {
                let count = arguments.len();
                unsafe {
                    let result = execute_function_with_context(
                        self.as_ptr(),
                        context.into_raw(),
                        object.map(|obj| obj.into_raw()).unwrap_or_else(null_mut),
                        count,
                        arguments
                            .iter()
                            .map(|v| v.clone().into_raw())
                            .collect::<Vec<_>>()
                            .as_ptr(),
                    );
                    V8Value::from_ptr(result)
                }
            })
    }
}

impl From<bool> for V8Value {
    /// Create a new V8Value object of type bool.
    fn from(value: bool) -> Self {
        unsafe { V8Value::from_ptr(cef_v8value_create_bool(value as i32)) }.unwrap()
    }
}

impl From<i32> for V8Value {
    /// Create a new V8Value object of type int.
    fn from(value: i32) -> Self {
        unsafe { V8Value::from_ptr(cef_v8value_create_int(value)) }.unwrap()
    }
}

impl From<u32> for V8Value {
    /// Create a new V8Value object of type unsigned int.
    fn from(value: u32) -> Self {
        unsafe { V8Value::from_ptr(cef_v8value_create_uint(value)) }.unwrap()
    }
}

impl From<f64> for V8Value {
    /// Create a new V8Value object of type double.
    fn from(value: f64) -> Self {
        unsafe { V8Value::from_ptr(cef_v8value_create_double(value)) }.unwrap()
    }
}

impl TryFrom<SystemTime> for V8Value {
    type Error = SystemTimeError;
    /// Create a new V8Value object of type Date. This function should only be
    /// called from within the scope of a [RenderProcessHandlerCallbacks],
    /// [V8Handler] or [V8AccessorHandler] callback, or in combination with calling
    /// [V8Context::enter] and [V8Context::exit] on a stored [V8Context] reference.
    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        let duration = value.duration_since(SystemTime::UNIX_EPOCH)?;
        let mut result = unsafe { std::mem::zeroed() };

        unsafe {
            cef_time_from_doublet(duration.as_secs_f64(), &mut result);
        } // This could fail in theory, but the actual implementation only returns 0 if the result is NULL
        Ok(unsafe { V8Value::from_ptr(cef_v8value_create_date(&result)) }.unwrap())
    }
}

impl From<&str> for V8Value {
    /// Create a new V8Value object of type string.
    fn from(value: &str) -> Self {
        let cefstr = CefString::new(value);
        unsafe { V8Value::from_ptr(cef_v8value_create_string(cefstr.as_ptr())) }.unwrap()
        // cef_v8value_t takes ownership of this string by copying the pointer to the data and the dtor to its own structure and nulling out ours, so our Drop implementation won't do anything.
    }
}

ref_counted_ptr!{
    pub struct V8Accessor(*mut cef_v8accessor_t);
}

impl V8Accessor {
    pub fn new<C: V8AccessorCallbacks>(callbacks: C) -> V8Accessor {
        unsafe{ V8Accessor::from_ptr_unchecked(V8AccessorWrapper::new(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Trait that should be implemented to handle V8 accessor calls. Accessor
/// identifiers are registered by calling [V8Value::set_value]. The
/// functions of this trait will be called on the thread associated with the
/// V8 accessor.
pub trait V8AccessorCallbacks: 'static + Send + Sync {
    /// Handle retrieval the accessor value identified by `name`. `object` is the
    /// receiver ('this' object) of the accessor. If retrieval succeeds return
    /// `Ok(retval)`. If retrieval fails return `Err(exception)` to be thrown
    /// as an exception.
    fn get(&self, name: &str, object: &V8Value) -> Result<V8Value, String>;
    /// Handle assignment of the accessor value identified by `name`. `object` is
    /// the receiver ('this' object) of the accessor. `value` is the new value
    /// being assigned to the accessor. If assignment fails return `Err(exception)`
    /// to be thrown as an exception.
    fn set(&self, name: &str, object: &V8Value, value: &V8Value) -> Result<(), String>;
}

pub(crate) struct V8AccessorWrapper(Mutex<Box<dyn V8AccessorCallbacks>>);

impl V8AccessorWrapper {
    pub(crate) fn new(delegate: Box<dyn V8AccessorCallbacks>) -> Self {
        Self(Mutex::new(delegate))
    }
}

impl Wrapper for V8AccessorWrapper {
    type Cef = cef_v8accessor_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_v8accessor_t {
                base: unsafe { std::mem::zeroed() },
                get: Some(Self::get),
                set: Some(Self::set),
            },
            self,
        )
    }
}
cef_callback_impl! {
    impl for V8AccessorWrapper: cef_v8accessor_t {
        fn get(
            &self,
            name:      &CefString             : *const cef_string_t,
            object:    V8Value                : *mut cef_v8value_t,
            retval:    *mut *mut cef_v8value_t: *mut *mut cef_v8value_t,
            exception: &mut CefString         : *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name: String = name.into();
            match self.0.lock().get(&name, &object) {
                Ok(value) => {
                    unsafe { (*retval) = value.into_raw(); }
                    1
                }
                Err(exception_str) => {
                    exception.set_string(&exception_str);
                    0
                },
            }
        }
        fn set(
            &self,
            name     : &CefString         : *const cef_string_t,
            object   : V8Value            : *mut cef_v8value_t,
            value    : V8Value            : *mut cef_v8value_t,
            exception: &mut CefString     : *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name: String = name.into();
            if let Err(exception_str) = self.0.lock().set(&name, &object, &value) {
                exception.set_string(&exception_str);
                0
            } else {
                1
            }
        }
    }
}

ref_counted_ptr!{
    pub struct V8Interceptor(*mut cef_v8interceptor_t);
}

impl V8Interceptor {
    pub fn new<C: V8InterceptorCallbacks>(callbacks: C) -> V8Interceptor {
        unsafe{ V8Interceptor::from_ptr_unchecked(V8InterceptorWrapper::new(Box::new(callbacks)).wrap().into_raw()) }
    }
}

/// Trait that should be implemented to handle V8 interceptor calls. The
/// functions of this trait will be called on the thread associated with the
/// V8 interceptor. Interceptor's named property handlers (with first argument of
/// type `&str`) are called when object is indexed by string. Indexed property
/// handlers (with first argument of type `i32`) are called when object is indexed
/// by integer.
pub trait V8InterceptorCallbacks: Sync + Send + 'static {
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
    fn set_byname(&self, name: &str, object: &V8Value, value: &V8Value) -> Result<(), String>;
    /// Handle assignment of the interceptor value identified by `index`. `object`
    /// is the receiver ('this' object) of the interceptor. `value` is the new
    /// value being assigned to the interceptor. If assignment fails, return
    /// `Err(_)` with the exception that will be thrown.
    fn set_byindex(&self, index: i32, object: &V8Value, value: &V8Value) -> Result<(), String>;
}

pub(crate) struct V8InterceptorWrapper(Mutex<RefCell<Box<dyn V8InterceptorCallbacks>>>);

impl V8InterceptorWrapper {
    pub(crate) fn new(delegate: Box<dyn V8InterceptorCallbacks>) -> Self {
        Self(Mutex::new(RefCell::new(delegate)))
    }
}

impl Wrapper for V8InterceptorWrapper {
    type Cef = cef_v8interceptor_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_v8interceptor_t {
                base: unsafe { std::mem::zeroed() },
                get_byname: Some(Self::get_byname),
                get_byindex: Some(Self::get_byindex),
                set_byname: Some(Self::set_byname),
                set_byindex: Some(Self::set_byindex),
            },
            self,
        )
    }
}

cef_callback_impl! {
    impl for V8InterceptorWrapper: cef_v8interceptor_t {
        fn get_byname(
            &self,
            name:      &CefString             : *const cef_string_t,
            object:    V8Value                : *mut cef_v8value_t,
            retval:    *mut *mut cef_v8value_t: *mut *mut cef_v8value_t,
            exception: &mut CefString         : *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name: String = name.into();
            if let Some(found) = self.0.lock().borrow().get_byname(&name, &object) {
                match found {
                    Ok(value) => {
                        unsafe { (*retval) = value.into_raw(); }
                        1
                    },
                    Err(exception_str) => {
                        exception.set_string(&exception_str);
                        0
                    },
                }
            } else {
                0
            }
        }
        fn get_byindex(
            &self,
            index:     i32                    : std::os::raw::c_int,
            object:    V8Value                : *mut cef_v8value_t,
            retval:    *mut *mut cef_v8value_t: *mut *mut cef_v8value_t,
            exception: &mut CefString         : *mut cef_string_t,
        ) -> std::os::raw::c_int {
            if let Some(found) = self.0.lock().borrow().get_byindex(index, &object) {
                match found {
                    Ok(value) => {
                        unsafe { (*retval) = value.into_raw(); }
                        1
                    },
                    Err(exception_str) => {
                        exception.set_string(&exception_str);
                        0
                    },
                }
            } else {
                0
            }
        }
        fn set_byname(
            &self,
            name     : &CefString    : *const cef_string_t,
            object   : V8Value       : *mut cef_v8value_t,
            value    : V8Value       : *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name: String = name.into();
            if let Err(exception_str) = self.0.lock().borrow_mut().set_byname(&name, &object, &value) {
                exception.set_string(&exception_str);
                0
            } else {
                1
            }
        }
        fn set_byindex(
            &self,
            index    : i32           : std::os::raw::c_int,
            object   : V8Value       : *mut cef_v8value_t,
            value    : V8Value       : *mut cef_v8value_t,
            exception: &mut CefString: *mut cef_string_t,
        ) -> std::os::raw::c_int {
            if let Err(exception_str) = self.0.lock().borrow_mut().set_byindex(index, &object, &value) {
                exception.set_string(&exception_str);
                0
            } else {
                1
            }
        }
    }
}

struct V8ArrayBufferReleaseCallbackWrapper(
    Mutex<Option<Box<dyn FnOnce(*mut u8) + Send + 'static>>>,
);

impl V8ArrayBufferReleaseCallbackWrapper {
    fn new(delegate: impl FnOnce(*mut u8) + Send + 'static) -> Self {
        Self(Mutex::new(Some(Box::new(delegate))))
    }
}

ref_counter!(cef_v8array_buffer_release_callback_t);
impl Wrapper for V8ArrayBufferReleaseCallbackWrapper {
    type Cef = cef_v8array_buffer_release_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_v8array_buffer_release_callback_t {
                base: unsafe { std::mem::zeroed() },
                release_buffer: Some(V8ArrayBufferReleaseCallbackWrapper::release_buffer),
            },
            self,
        )
    }
}

cef_callback_impl! {
    impl for V8ArrayBufferReleaseCallbackWrapper: cef_v8array_buffer_release_callback_t {
        fn release_buffer(&self, buffer: *mut std::os::raw::c_void: *mut std::os::raw::c_void) {
            if let Some(release) = self.0.lock().take() {
                release(buffer as *mut u8);
            }
        }
    }
}

struct V8HandlerWrapper(
    Box<dyn Fn(&str, V8Value, &[V8Value]) -> Result<V8Value, String> + Sync + Send + 'static>,
);

impl V8HandlerWrapper {
    fn new(
        delegate: Box<
            dyn Fn(&str, V8Value, &[V8Value]) -> Result<V8Value, String> + Sync + Send + 'static,
        >,
    ) -> Self {
        Self(delegate)
    }
}

impl Wrapper for V8HandlerWrapper {
    type Cef = cef_v8handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_v8handler_t {
                base: unsafe { std::mem::zeroed() },
                execute: Some(Self::execute),
            },
            self,
        )
    }
}

cef_callback_impl! {
    impl for V8HandlerWrapper: cef_v8handler_t {
        fn execute(
            &self,
            name           : &CefString                : *const cef_string_t,
            object         : V8Value                   : *mut cef_v8value_t,
            arguments_count: usize                     : usize,
            arguments      : *const *mut cef_v8value_t : *const *mut cef_v8value_t,
            retval         : &mut *mut cef_v8value_t   : *mut *mut cef_v8value_t,
            exception      : &mut CefString            : *mut cef_string_t,
        ) -> std::os::raw::c_int {
            let name: String = name.into();
            let args: Vec<V8Value> = unsafe { std::slice::from_raw_parts(arguments, arguments_count) }.iter().map(|val| unsafe { V8Value::from_ptr_unchecked(*val) }).collect();
            match self.0(&name, object, &args) {
                Ok(value) => {
                    *retval = value.into_raw();
                    1
                }
                Err(err) => {
                    exception.set_string(&err);
                    0
                }
            }
        }
    }
}

ref_counted_ptr! {
    pub struct V8Handler(*mut cef_v8handler_t);
}

impl V8Handler {
    pub fn execute(
        &self,
        name: &str,
        object: V8Value,
        arguments: &[V8Value],
    ) -> Result<V8Value, String> {
        self.0
            .execute
            .ok_or_else(|| "Execute is null".to_owned())
            .and_then(|execute| {
                let mut retval = null_mut();
                let mut exception = CefString::null();
                if unsafe {
                    execute(
                        self.as_ptr(),
                        CefString::new(name).as_ptr(),
                        object.as_ptr(),
                        arguments.len(),
                        arguments
                            .iter()
                            .map(|v| v.as_ptr())
                            .collect::<Vec<_>>()
                            .as_ptr(),
                        &mut retval,
                        exception.as_ptr_mut(),
                    ) != 0
                } {
                    Ok(unsafe { V8Value::from_ptr_unchecked(retval) })
                } else {
                    Err(exception.into())
                }
            })
    }
}

impl Fn<(&str, V8Value, &[V8Value])> for V8Handler {
    extern "rust-call" fn call(
        &self,
        (name, object, arguments): (&str, V8Value, &[V8Value]),
    ) -> Result<V8Value, String> {
        self.execute(name, object, arguments)
    }
}

impl FnMut<(&str, V8Value, &[V8Value])> for V8Handler {
    extern "rust-call" fn call_mut(
        &mut self,
        (name, object, arguments): (&str, V8Value, &[V8Value]),
    ) -> Result<V8Value, String> {
        self.execute(name, object, arguments)
    }
}

impl FnOnce<(&str, V8Value, &[V8Value])> for V8Handler {
    type Output = Result<V8Value, String>;
    extern "rust-call" fn call_once(
        self,
        (name, object, arguments): (&str, V8Value, &[V8Value]),
    ) -> Self::Output {
        self.execute(name, object, arguments)
    }
}

#[doc(hidden)]
pub struct CefUserData {
    base: cef_base_ref_counted_t,
}

ref_counted_ptr!{
    /// User Data wrapper used for storing in V8Value objects that takes care of
    /// memory management between CEF and Rust.
    pub struct UserData(*mut CefUserData);
}

impl UserData {
    pub fn new<T: Any + Sync + Send>(data: T) -> UserData {
        unsafe{ UserData::from_ptr_unchecked(UserDataInner(Box::new(data)).wrap().into_raw()) }
    }
}

struct UserDataInner(Box<dyn Any + Sync + Send>);

impl Wrapper for UserDataInner {
    type Cef = CefUserData;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            unsafe{ std::mem::zeroed() },
            self,
        )
    }
}

impl std::ops::Deref for UserData {
    type Target = dyn Any + Send + Sync;
    fn deref(&self) -> &Self::Target {
        unsafe{ crate::refcounted::RefCounted::<UserDataInner>::wrapper(self.as_ptr()) }
    }
}
