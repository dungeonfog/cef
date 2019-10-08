use cef_sys::cef_main_args_t;
use winapi::shared::minwindef::HINSTANCE;
use std::ffi::CString;

/// Structure representing CefExecuteProcess arguments.
pub struct MainArgs {
    cef: cef_main_args_t,
    #[cfg(not(target_os = "windows"))]
    rust: Vec<*mut c_char>,
}

impl MainArgs {
    /// Create the main arguments object for Windows.
    /// `instance` is the base address of the module in memory, as provided by the Windows API.
    /// You can use `GetModuleHandleA` from the winapi crate (passing a NULL pointer) to get this.
    #[cfg(target_os = "windows")]
    pub fn new(instance: HINSTANCE) -> Self {
        Self {
            cef: cef_main_args_t {
                instance,
            }
        }
    }
    /// Create the main arguments for Linux and Mac.
    /// `args` are the command line arguments. A good place to start is [std::env::args].
    #[cfg(not(target_os = "windows"))]
    pub fn new<I: IntoIterator<Item = String>>(args: I) {
        let mut args: Vec<*mut c_char> = args.into_iter().map(|arg| CString::new(arg).unwrap().into_raw()).collect();
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
