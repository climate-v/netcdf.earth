#[macro_use]
extern crate lazy_static;

use std::os::raw::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use netcdf::*;
use std::sync::Mutex;


fn main() {
}

fn from_js_str(str_prt: *mut c_char) -> String {
    unsafe {
        CStr::from_ptr(str_prt).to_string_lossy().into_owned()
    }
}

fn to_cstr(string: &str) -> *mut c_char {
    CString::new(string).unwrap().into_raw()
}

lazy_static! {
    static ref FILE_MUTEX: Mutex<Option<netcdf::File>> = Mutex::new(None);
}

#[no_mangle]
pub extern fn get_title() -> *mut c_char {
    let mut guard = FILE_MUTEX.lock().unwrap();
    match &mut *guard {
        None => return to_cstr(""),
        Some(file) => {
            let attribute_title = file.attribute("title");
            match attribute_title {
                Some(attribute) => {
                    match attribute.value().unwrap() {
                        AttrValue::Str(x) => return to_cstr(&x),
                        _ => return to_cstr("wrong attr type")
                    }
                },
                None => return to_cstr("no title attr")
            }
        }
    }
}

#[no_mangle]
pub extern fn get_variables() -> *mut c_char {
    let guard = FILE_MUTEX.lock().unwrap();
    match &*guard {
        None => return to_cstr(""),
        Some(file) => {
            let variables = file.variables();
            let mut return_string = String::new();
            for var in variables {
                if !return_string.is_empty() {
                    return_string.push(',');
                }

                return_string.push_str(&var.name());
            }

            return to_cstr(&return_string);
        }
    }
}

#[no_mangle]
pub extern fn get_dimensions() -> *mut c_char {
    let guard = FILE_MUTEX.lock().unwrap();
    match &*guard {
        None => return to_cstr(""),
        Some(file) => {
            let dimensions = file.dimensions();
            let mut return_string = String::new();
            for dimension in dimensions {
                if !return_string.is_empty() {
                    return_string.push(',');
                }

                return_string.push_str(&dimension.name());
            }

            return to_cstr(&return_string);
        }
    }
}

#[no_mangle]
pub extern fn get_variable_dimensions(name: *mut c_char) -> *mut c_char {
    let guard = FILE_MUTEX.lock().unwrap();
    let variable_name = from_js_str(name);
    match &*guard {
        None => return to_cstr(""),
        Some(file) => {
            match file.variable(&variable_name) {
                Some(variable) => {
                    let dimensions = variable.dimensions();
                    let mut return_string = String::new();
                    for dimension in dimensions {
                        if !return_string.is_empty() {
                            return_string.push(',');
                        }

                        return_string.push_str(&dimension.name());
                    }

                    return to_cstr(&return_string);
                }
                _ => return to_cstr("")
            }
        }
    }
}

#[no_mangle]
pub extern fn open_file(name: *mut c_char) -> bool {
    let file_name = from_js_str(name);
    let mut guard = FILE_MUTEX.lock().unwrap();
    match netcdf::open(file_name) {
        Ok(file) => {
            *guard = Some(file);
            return true;
        },
        Err(_) => return false
    }
}

#[no_mangle]
pub extern fn close_file() {
    let mut guard = FILE_MUTEX.lock().unwrap();
    match &*guard {
        Some(_file) => {
            *guard = None;
        }
        _ => {}
    }
}