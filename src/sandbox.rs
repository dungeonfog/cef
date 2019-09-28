use cef_sys::{cef_sandbox_info_create, cef_sandbox_info_destroy};

// The sandbox is used to restrict sub-processes (renderer, plugin, GPU, etc)
// from directly accessing system resources. This helps to protect the user
// from untrusted and potentially malicious Web content.
// See http://www.chromium.org/developers/design-documents/sandbox for
// complete details.
//
// To enable the sandbox on Windows the following requirements must be met:
// 1. Use the same executable for the browser process and all sub-processes.
// 2. Link the executable with the cef_sandbox static library.
// 3. Call the [SandboxInfo::new] function from within the executable
//    (not from a separate DLL) and pass the resulting pointer into both the
//    [App::execute_process] and [App::initialize] functions via the
//    `windows_sandbox_info` parameter.
pub struct SandboxInfo(*mut std::os::raw::c_void);

impl SandboxInfo {
    /// Create the sandbox information object for this process. It is safe to create
    /// multiple of this object and to drop the object immediately after passing
    /// into the [App::execute_process] and/or [App::initialize] functions.
    pub fn new() -> Self {
        Self(unsafe { cef_sandbox_info_create() })
    }
    pub(crate) fn get(&self) -> *mut std::os::raw::c_void {
        self.0
    }
}

impl Drop for SandboxInfo {
    /// Destroy the specified sandbox information object.
    fn drop(&mut self) {
        unsafe { cef_sandbox_info_destroy(self.0); }
    }
}
