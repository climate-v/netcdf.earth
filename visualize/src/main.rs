#[macro_use]
extern crate lazy_static;

use std::os::raw::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use netcdf::*;
use std::sync::Mutex;
use std::mem;
use paste::paste;

#[repr(C)]
#[derive(Debug)]
pub struct JsBytes {
    ptr: u32,
    len: u32,
    cap: u32,
}

impl JsBytes {
    pub fn new<T>(mut bytes: Vec<T>) -> *mut JsBytes {
        let ptr = bytes.as_mut_ptr() as u32;
        let len = bytes.len() as u32;
        let cap = bytes.capacity() as u32;
        mem::forget(bytes);
        let boxed = Box::new(JsBytes { ptr, len, cap });
        Box::into_raw(boxed)
    }
}

#[no_mangle]
pub fn drop_bytes(ptr: *mut JsBytes) {
    unsafe {
        let boxed: Box<JsBytes> = Box::from_raw(ptr);
        Vec::from_raw_parts(boxed.ptr as *mut u32, boxed.len as usize, boxed.cap as usize);
    }
}

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

fn to_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    unsafe {
        return std::slice::from_raw_parts(ptr, len);
    }
}

lazy_static! {
    static ref FILE_MUTEX: Mutex<Option<netcdf::File>> = Mutex::new(None);
}

#[no_mangle]
pub extern fn get_string_attribute(name: *mut c_char) -> *mut c_char {
    let attribute_name = from_js_str(name);
    let mut guard = FILE_MUTEX.lock().unwrap();
    match &mut *guard {
        None => return to_cstr(""),
        Some(file) => {
            match file.attribute(&attribute_name) {
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
pub extern fn get_variable_type(name: *mut c_char) -> *mut c_char {
    let variable_name = from_js_str(name);
    let guard = FILE_MUTEX.lock().unwrap();
    match &*guard {
        None => return to_cstr(""),
        Some(file) => {
            match file.variable(&variable_name) {
                Some(variable) => {
                    return to_cstr(&variable.vartype().name())
                },
                _ => return to_cstr("")
            }
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

macro_rules! get_values_def {
    ($x:ty) => {
        paste! {
            #[no_mangle]
            pub extern fn [<get_ $x _values_for>](name: *mut c_char, data: *const i32, data_len: usize, len: *const i32, len_len: usize) -> *mut JsBytes {
                let guard = FILE_MUTEX.lock().unwrap();
                let variable_name = from_js_str(name);
                let start_values = to_slice(data, data_len); // We need to do this anyway so that if we drop it, we free the memory too
                let end_values = to_slice(len, len_len);
                let start_indices: Vec<usize> = match data_len {
                    0 => Vec::new(),
                    _ => start_values.to_vec().into_iter().map(|v| v as usize).collect()
                };
            
                let end_indices: Vec<usize> = match len_len {
                    0 => Vec::new(),
                    _ => end_values.to_vec().into_iter().map(|v| v as usize).collect()
                };
            
                match &*guard {
                    Some(file) => {
                        let variable = file.variable(&variable_name).unwrap();
                        let start = match start_indices.len() {
                            0 => None,
                            _ => Some(start_indices.as_slice())
                        };
            
                        let end = match end_indices.len() {
                            0 => None,
                            _ => Some(end_indices.as_slice())
                        };

                        let values = variable.values::<$x>(start, end).unwrap();
                        let value_vec = values.into_raw_vec();
                        return JsBytes::new(value_vec);
                    }
                    _ => {
                        return JsBytes::new::<$x>(Vec::new());
                    }
                }
            }
        }
    }
}

get_values_def!(i8);
get_values_def!(i16);
get_values_def!(i32);
get_values_def!(f32);
get_values_def!(f64);
get_values_def!(u8);
get_values_def!(u16);
get_values_def!(u32);

macro_rules! get_value_def {
    ($x:ty, $y:literal) => {
        paste! {
            #[no_mangle]
            pub extern fn [<get_ $x _value_for>](name: *mut c_char, data: *const i32, data_len: usize) -> $x {
                let guard = FILE_MUTEX.lock().unwrap();
                let variable_name = from_js_str(name);
                let index_values = to_slice(data, data_len); // We need to do this anyway so that if we drop it, we free the memory too
                let indices: Vec<usize> = index_values.to_vec().into_iter().map(|v| v as usize).collect();
            
                match &*guard {
                    Some(file) => {
                        let variable = file.variable(&variable_name).unwrap();
                        let value = variable.value::<$x>(Some(indices.as_slice())).unwrap();
                        return value;
                    }
                    _ => return $y
                }
            }
        }
    }
}

get_value_def!(i8, 0);
get_value_def!(i16, 0);
get_value_def!(i32, 0);
get_value_def!(f32, 0.0);
get_value_def!(f64, 0.0);
get_value_def!(u8, 0);
get_value_def!(u16, 0);
get_value_def!(u32, 0);