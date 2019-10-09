use cef_sys::{
    cef_command_line_create, cef_command_line_get_global, cef_command_line_t, cef_string_map_alloc,
    cef_string_map_free, cef_string_map_key, cef_string_map_size, cef_string_map_value,
    cef_string_userfree_utf16_free,
};
use std::{collections::HashMap, ptr::null_mut};
#[cfg(not(target_os = "windows"))]
use std::{ffi::CString, os::raw::c_char};

use crate::{
    app::App,
    string::{CefString, CefStringList},
};

/// Structure used to create and/or parse command line arguments. Arguments with
/// `--`, `-` and, on Windows, `/` prefixes are considered switches. Switches
/// will always precede any arguments without switch prefixes. Switches can
/// optionally have a value specified using the `=` delimiter (e.g.
/// `-switch=value`). An argument of `--` will terminate switch parsing with all
/// subsequent tokens, regardless of prefix, being interpreted as non-switch
/// arguments. Switch names are considered case-insensitive. This structure can
/// be used before [App::initialize] is called.
pub struct CommandLine(*mut cef_command_line_t);

impl CommandLine {
    /// Returns the singleton global CommandLine object. The returned object
    /// will be read-only.
    pub fn get_global() -> Self {
        Self(unsafe { cef_command_line_get_global() })
    }
    /// Create a new CommandLine instance.
    pub fn new() -> Self {
        Self(unsafe { cef_command_line_create() })
    }

    /// Returns true if this object is valid. Do not call any other functions
    /// if this function returns false.
    pub fn is_valid(&self) -> bool {
        self.as_ref()
            .is_valid
            .and_then(|is_valid| Some(unsafe { is_valid(self.0) } != 0))
            .unwrap_or(false)
    }
    /// Returns true if the values of this object are read-only. Some APIs may
    /// expose read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.as_ref()
            .is_read_only
            .and_then(|is_read_only| Some(unsafe { is_read_only(self.0) } != 0))
            .unwrap_or(true)
    }
    /// Initialize the command line with the specified |argv| values.
    /// The first argument must be the name of the program. This function is only
    /// supported on non-Windows platforms.
    #[cfg(not(target_os = "windows"))]
    pub fn new_from_argv(argv: &[&str]) -> Self {
        let instance = unsafe { cef_command_line_create() };
        let argv: Vec<*const c_char> = argv
            .iter()
            .map(|arg| CString::new(*arg).unwrap().as_ptr())
            .collect();

        unsafe {
            ((*instance).init_from_argv.unwrap())(instance, argv.len() as i32, argv.as_ptr());
        }
        Self(instance)
    }
    /// Initialize the command line with the string returned by calling
    /// GetCommandLineW(). This function is only supported on Windows.
    pub fn new_from_string(command_line: &str) -> Self {
        let instance = unsafe { cef_command_line_create() };
        unsafe {
            ((*instance).init_from_string.unwrap())(
                instance,
                CefString::new(command_line).as_ref(),
            );
        }
        Self(instance)
    }
    /// Reset the command-line switches and arguments but leave the program
    /// component unchanged.
    pub fn reset(&mut self) {
        unsafe {
            (self.as_ref().reset.unwrap())(self.0);
        }
    }
    /// Retrieve the original command line string as a vector of strings. The argv
    /// array: `{ program, [(--|-|/)switch[=value]]*, [--], [argument]* }`
    pub fn get_argv(&self) -> Vec<String> {
        let list = CefStringList::new();
        unsafe {
            (self.as_ref().get_argv.unwrap())(self.0, list.get());
        }
        list.into()
    }
    /// Constructs and returns the represented command line string. Use this
    /// function cautiously because quoting behavior is unclear.
    pub fn get_command_line_string(&self) -> String {
        let command_line = unsafe { (self.as_ref().get_command_line_string.unwrap())(self.0) };

        let command_line_str = String::from_utf16_lossy(unsafe {
            std::slice::from_raw_parts((*command_line).str, (*command_line).length)
        });

        unsafe {
            cef_string_userfree_utf16_free(command_line);
        }
        command_line_str
    }
    /// Get the program part of the command line string (the first item).
    pub fn get_program(&self) -> String {
        let program = unsafe { (self.as_ref().get_program.unwrap())(self.0) };
        let program_str = String::from_utf16_lossy(unsafe {
            std::slice::from_raw_parts((*program).str, (*program).length)
        });

        unsafe {
            cef_string_userfree_utf16_free(program);
        }
        program_str
    }
    /// Set the program part of the command line string (the first item).
    pub fn set_program(&mut self, program: &str) {
        let program = CefString::new(program);
        unsafe {
            (self.as_ref().set_program.unwrap())(self.0, program.as_ref());
        }
    }
    /// Returns true if the command line has switches.
    pub fn has_switches(&self) -> bool {
        unsafe { (self.as_ref().has_switches.unwrap())(self.0) != 0 }
    }
    /// Returns true if the command line contains the given switch.
    pub fn has_switch(&self, name: &str) -> bool {
        unsafe { (self.as_ref().has_switch.unwrap())(self.0, CefString::new(name).as_ref()) != 0 }
    }
    /// Returns the value associated with the given switch. If the switch has no
    /// value or isn't present this function returns None.
    pub fn get_switch_value(&self, name: &str) -> Option<String> {
        let value = unsafe {
            (self.as_ref().get_switch_value.unwrap())(self.0, CefString::new(name).as_ref())
        };
        if value.is_null() {
            return None;
        }
        let value_str = String::from_utf16_lossy(unsafe {
            std::slice::from_raw_parts((*value).str, (*value).length)
        });

        unsafe {
            cef_string_userfree_utf16_free(value);
        }
        Some(value_str)
    }
    /// Returns the map of switch names and values. If a switch has no value
    /// None is returned.
    pub fn get_switches(&self) -> HashMap<String, Option<String>> {
        let switches = unsafe { cef_string_map_alloc() };
        unsafe {
            (self.as_ref().get_switches.unwrap())(self.0, switches);
        }

        let result = (0..unsafe { cef_string_map_size(switches) })
            .map(|index| {
                let pair = (null_mut(), null_mut());
                unsafe {
                    cef_string_map_key(switches, index, pair.0);
                    cef_string_map_value(switches, index, pair.1);
                }
                (
                    CefString::copy_raw_to_string(pair.0).unwrap(),
                    CefString::copy_raw_to_string(pair.1),
                )
            })
            .collect();

        unsafe {
            cef_string_map_free(switches);
        }
        result
    }
    /// Add a switch to the end of the command line.
    pub fn append_switch(&mut self, name: &str) {
        unsafe {
            (self.as_ref().append_switch.unwrap())(self.0, CefString::new(name).as_ref());
        }
    }
    /// Add a switch with the specified value to the end of the command line.
    pub fn append_switch_with_value(&mut self, name: &str, value: &str) {
        unsafe {
            (self.as_ref().append_switch_with_value.unwrap())(
                self.0,
                CefString::new(name).as_ref(),
                CefString::new(value).as_ref(),
            );
        }
    }
    /// True if there are remaining command line arguments.
    pub fn has_arguments(&self) -> bool {
        unsafe { (self.as_ref().has_arguments.unwrap())(self.0) != 0 }
    }
    /// Get the remaining command line arguments.
    pub fn get_arguments(&self) -> Vec<String> {
        let list = CefStringList::new();
        unsafe {
            (self.as_ref().get_arguments.unwrap())(self.0, list.get());
        }
        list.into()
    }
    /// Add an argument to the end of the command line.
    pub fn append_argument(&mut self, argument: &str) {
        unsafe {
            (self.as_ref().append_argument.unwrap())(self.0, CefString::new(argument).as_ref());
        }
    }
    /// Insert a command before the current command. Common for debuggers, like
    /// "valgrind" or "`gdb --args`".
    pub fn prepend_wrapper(&mut self, wrapper: &str) {
        unsafe {
            (self.as_ref().prepend_wrapper.unwrap())(self.0, CefString::new(wrapper).as_ref());
        }
    }
}

#[doc(hidden)]
impl std::convert::AsRef<cef_command_line_t> for CommandLine {
    fn as_ref(&self) -> &cef_command_line_t {
        unsafe { self.0.as_ref().unwrap() }
    }
}

#[doc(hidden)]
impl From<*mut cef_command_line_t> for CommandLine {
    fn from(cmd: *mut cef_command_line_t) -> Self {
        unsafe {
            ((*cmd).base.add_ref.unwrap())(&mut (*cmd).base);
        }
        Self(cmd)
    }
}

impl Clone for CommandLine {
    /// Returns a writable copy of this object.
    fn clone(&self) -> Self {
        Self(unsafe { (self.as_ref().copy.unwrap())(self.0) })
    }
}

impl Drop for CommandLine {
    fn drop(&mut self) {
        unsafe {
            (self.as_ref().base.release.unwrap())(&mut (*self.0).base);
        }
    }
}
