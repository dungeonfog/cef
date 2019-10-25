use cef_sys::cef_main_args_t;

/// Structure representing CefExecuteProcess arguments.
pub struct MainArgs {
    cef: cef_main_args_t,
    #[cfg(not(target_os = "windows"))]
    rust: Vec<*mut c_char>,
}

impl MainArgs {
    /// Create the main arguments object.
    pub fn new() -> Self {
        Self::new_inner()
    }

    #[cfg(target_os = "windows")]
    fn new_inner() -> Self {
        use winapi::um::libloaderapi::GetModuleHandleW;
        let instance = unsafe{ GetModuleHandleW(std::ptr::null()) };
        Self {
            cef: cef_main_args_t { instance },
        }
    }
    #[cfg(not(target_os = "windows"))]
    fn new_inner() {
        let mut args: Vec<*mut c_char> = std::env::args_os()
            .map(|arg| CString::new(arg).unwrap().into_raw())
            .collect();
        Self {
            cef: Box::into_raw(Box::new(cef_main_args_t {
                argc: args.len() as i32,
                argv: args.as_mut_ptr(),
            })),
            rust: args,
        }
    }

    pub(crate) fn get(&self) -> *const cef_main_args_t {
        &self.cef
    }
    pub(crate) fn get_mut(&mut self) -> *mut cef_main_args_t {
        &mut self.cef
    }
}

impl Drop for MainArgs {
    fn drop(&mut self) {
        #[cfg(not(target_os = "windows"))]
        self.rust.into_iter().map(CString::from_raw);
    }
}
