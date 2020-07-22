use std::{io, ffi::CString, path::{Path, PathBuf}};
use lazy_static::lazy_static;
use parking_lot::Mutex;

/// On macos, loads CEF at runtime based on the given framework directory path.
///
/// If no path is provided, we look in a series of default paths to try to find CEF. Returns the
/// path that ends up getting used.
///
/// This will be automatically called by [`Context::initialize`](crate::Context::initialize).
/// However, if you need to call CEF functions before initializing the app, you should call this
/// function before calling any other functions.
pub fn load_framework(framework_dir_path: Option<&Path>) -> Result<PathBuf, io::Error> {
    let framework_path = framework_path_with_fallbacks(framework_dir_path)?;
    {
        let mut loader = FRAMEWORK_LOADER.lock();
        if loader.is_none() {
             *loader = unsafe{ Some(FrameworkLoader::new(&framework_path)?) };
        }
    }
    Ok(framework_path.parent().expect("framework path needs to have parent").to_owned())
}

pub fn framework_is_loaded() -> bool {
    FRAMEWORK_LOADER.lock().is_some()
}

fn framework_path_with_fallbacks(framework_dir_path: Option<&Path>) -> Result<PathBuf, io::Error> {
    let framework_path = framework_dir_path
       .and_then(|d| Some(d.display().to_string()).filter(|p| std::path::Path::new(p).exists()))
       .or_else(|| Some("../Frameworks/Chromium Embedded Framework.framework".to_owned()).filter(|p| std::path::Path::new(p).exists()))
       .or_else(|| cef_sys::MACOS_FRAMEWORK_PATH.map(|s| s.to_owned()).filter(|p| std::path::Path::new(p).exists()))
       .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find CEF framework path."))?;
    Ok(PathBuf::from(framework_path).canonicalize()?.join("Chromium Embedded Framework"))
}

struct FrameworkLoader {
    _marker: ()
}

impl FrameworkLoader {
    unsafe fn new(framework_path: &Path) -> Result<FrameworkLoader, io::Error> {
        let framework_path_cstr = CString::new(format!("{}", framework_path.display()))?;
        assert!(1 == cef_sys::cef_load_library(framework_path_cstr.as_ptr()));
        Ok(FrameworkLoader {
             _marker: ()
        })
    }
}

impl Drop for FrameworkLoader {
    fn drop(&mut self) {
        unsafe {
            cef_sys::cef_unload_library();
        }
    }
}

lazy_static!{
    static ref FRAMEWORK_LOADER: Mutex<Option<FrameworkLoader>> = Mutex::new(None);
}
