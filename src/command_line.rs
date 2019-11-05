use cef_sys::{
    cef_command_line_create, cef_command_line_get_global, cef_command_line_t, cef_string_map_alloc,
    cef_string_map_free, cef_string_map_key, cef_string_map_size, cef_string_map_value,
    cef_string_userfree_utf16_free,
};
use std::{collections::HashMap, ptr::null_mut};
#[cfg(not(target_os = "windows"))]
use std::{ffi::CString, os::raw::c_char};

use crate::string::{CefString, CefStringList};

ref_counted_ptr! {
    /// Structure used to create and/or parse command line arguments. Arguments with
    /// `--`, `-` and, on Windows, `/` prefixes are considered switches. Switches
    /// will always precede any arguments without switch prefixes. Switches can
    /// optionally have a value specified using the `=` delimiter (e.g.
    /// `-switch=value`). An argument of `--` will terminate switch parsing with all
    /// subsequent tokens, regardless of prefix, being interpreted as non-switch
    /// arguments. Switch names are considered case-insensitive. This structure can
    /// be used before [App::initialize] is called.
    pub struct CommandLine(*mut cef_command_line_t);
}

impl CommandLine {
    /// Returns the singleton global CommandLine object. The returned object
    /// will be read-only.
    pub fn get_global() -> Self {
        unsafe { CommandLine::from_ptr_unchecked(cef_command_line_get_global()) }
    }
    /// Create a new CommandLine instance.
    pub fn new() -> Self {
        unsafe { CommandLine::from_ptr_unchecked(cef_command_line_create()) }
    }

    /// Returns true if this object is valid. Do not call any other functions
    /// if this function returns false.
    /// TODO: IF `is_valid` IS FALSE, DOES CALLING OTHER FUNCTIONS VIOLATE SAFETY --OSSPIAL
    pub fn is_valid(&self) -> bool {
        self.0
            .is_valid
            .map(|is_valid| unsafe { is_valid(self.as_ptr()) } != 0)
            .unwrap_or(false)
    }
    /// Returns true if the values of this object are read-only. Some APIs may
    /// expose read-only objects.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) } != 0)
            .unwrap_or(true)
    }
    /// Initialize the command line with the specified |argv| values.
    /// The first argument must be the name of the program. This function is only
    /// supported on non-Windows platforms.
    #[cfg(not(target_os = "windows"))]
    pub fn new_from_argv(argv: &[&str]) -> Self {
        let instance = Self::new();
        let argv: Vec<*const c_char> = argv
            .iter()
            .map(|arg| CString::new(*arg).unwrap().as_ptr())
            .collect();

        unsafe {
            (instance.0.init_from_argv.unwrap())(instance.as_ptr(), argv.len() as i32, argv.as_ptr());
        }
        instance
    }
    /// Initialize the command line with the string returned by calling
    /// GetCommandLineW(). This function is only supported on Windows.
    pub fn new_from_string(command_line: &str) -> Self {
        let instance = CommandLine::new();
        unsafe {
            (instance.0.init_from_string.unwrap())(
                instance.as_ptr(),
                CefString::new(command_line).as_ptr(),
            );
        }
        instance
    }
    /// Reset the command-line switches and arguments but leave the program
    /// component unchanged.
    pub fn reset(&self) {
        unsafe {
            (self.0.reset.unwrap())(self.as_ptr());
        }
    }
    /// Retrieve the original command line string as a vector of strings. The argv
    /// array: `{ program, [(--|-|/)switch[=value]]*, [--], [argument]* }`
    pub fn get_argv(&self) -> Vec<String> {
        let mut list = CefStringList::new();
        unsafe {
            (self.0.get_argv.unwrap())(self.as_ptr(), list.as_mut_ptr());
        }
        list.into()
    }
    /// Constructs and returns the represented command line string. Use this
    /// function cautiously because quoting behavior is unclear.
    pub fn get_command_line_string(&self) -> String {
        let command_line = unsafe { (self.0.get_command_line_string.unwrap())(self.as_ptr()) };

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
        let program = unsafe { (self.0.get_program.unwrap())(self.as_ptr()) };
        let program_str = String::from_utf16_lossy(unsafe {
            std::slice::from_raw_parts((*program).str, (*program).length)
        });

        unsafe {
            cef_string_userfree_utf16_free(program);
        }
        program_str
    }
    /// Set the program part of the command line string (the first item).
    pub fn set_program(&self, program: &str) {
        let program = CefString::new(program);
        unsafe {
            (self.0.set_program.unwrap())(self.as_ptr(), program.as_ptr());
        }
    }
    /// Returns true if the command line has switches.
    pub fn has_switches(&self) -> bool {
        unsafe { (self.0.has_switches.unwrap())(self.as_ptr()) != 0 }
    }
    /// Returns true if the command line contains the given switch.
    pub fn has_switch(&self, name: &str) -> bool {
        unsafe { (self.0.has_switch.unwrap())(self.as_ptr(), CefString::new(name).as_ptr()) != 0 }
    }
    /// Returns the value associated with the given switch. If the switch has no
    /// value or isn't present this function returns None.
    pub fn get_switch_value(&self, name: &str) -> Option<String> {
        let value = unsafe {
            (self.0.get_switch_value.unwrap())(self.as_ptr(), CefString::new(name).as_ptr())
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
            (self.0.get_switches.unwrap())(self.as_ptr(), switches);
        }

        let result = (0..unsafe { cef_string_map_size(switches) })
            .map(|index| {
                let pair = (null_mut(), null_mut());
                unsafe {
                    cef_string_map_key(switches, index, pair.0);
                    cef_string_map_value(switches, index, pair.1);
                    (
                        String::from(CefString::from_ptr_unchecked(pair.0)),
                        CefString::from_ptr(pair.1).map(String::from),
                    )
                }
            })
            .collect();

        unsafe {
            cef_string_map_free(switches);
        }
        result
    }
    /// Add a switch to the end of the command line.
    pub fn append_switch(&self, name: &str) {
        unsafe {
            (self.0.append_switch.unwrap())(self.as_ptr(), CefString::new(name).as_ptr());
        }
    }
    /// Add a switch with the specified value to the end of the command line.
    pub fn append_switch_with_value(&self, name: &str, value: &str) {
        unsafe {
            (self.0.append_switch_with_value.unwrap())(
                self.as_ptr(),
                CefString::new(name).as_ptr(),
                CefString::new(value).as_ptr(),
            );
        }
    }
    /// True if there are remaining command line arguments.
    pub fn has_arguments(&self) -> bool {
        unsafe { (self.0.has_arguments.unwrap())(self.as_ptr()) != 0 }
    }
    /// Get the remaining command line arguments.
    pub fn get_arguments(&self) -> Vec<String> {
        let mut list = CefStringList::new();
        unsafe {
            (self.0.get_arguments.unwrap())(self.as_ptr(), list.as_mut_ptr());
        }
        list.into()
    }
    /// Add an argument to the end of the command line.
    pub fn append_argument(&self, argument: &str) {
        unsafe {
            (self.0.append_argument.unwrap())(self.as_ptr(), CefString::new(argument).as_ptr());
        }
    }
    /// Insert a command before the current command. Common for debuggers, like
    /// "valgrind" or "`gdb --args`".
    pub fn prepend_wrapper(&self, wrapper: &str) {
        unsafe {
            (self.0.prepend_wrapper.unwrap())(self.as_ptr(), CefString::new(wrapper).as_ptr());
        }
    }
}

impl Default for CommandLine {
    fn default() -> Self {
        Self::new()
    }
}
impl crate::cef_helper_traits::DeepClone for CommandLine {
    /// Returns a writable copy of this object.
    fn deep_clone(&self) -> CommandLine {
        unsafe { Self::from_ptr_unchecked(self.0.copy.unwrap()(self.as_ptr())) }
    }
}
