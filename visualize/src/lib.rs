use crate::error::ToJs;
use crate::utils::set_panic_hook;
use crate::wasm_file::WasmFile;
use crate::wrapper::new_wrapper;
use headscratcher::NetCDF;
use wasm_bindgen::prelude::*;
use web_sys::Blob;
use wrapper::NetCDFHandle;

mod error;
mod utils;
mod wasm_file;
mod wrapper;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn load_file(blob: Blob) -> Result<NetCDFHandle, JsValue> {
    set_panic_hook();
    let wasm_file = WasmFile::new(blob);
    let netcdf = match NetCDF::new_from_file(wasm_file) {
        Ok(file) => file,
        Err(hse) => return Err(hse.into_js()),
    };
    Ok(new_wrapper(netcdf))
}

#[wasm_bindgen]
pub fn load_remote(url: String, size: usize) -> Result<NetCDFHandle, JsValue> {
    set_panic_hook();
    let wasm_file = WasmFile::new_remote(url, size as u64);
    let netcdf = match NetCDF::new_from_file(wasm_file) {
        Ok(file) => file,
        Err(hse) => return Err(hse.into_js()),
    };
    Ok(new_wrapper(netcdf))
}
