use crate::error::ToJs;
use crate::wasm_file::WasmFile;
use headscratcher::error::HeadScratcherError;
use headscratcher::parser::components::NetCDFType;
use headscratcher::NetCDF;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize)]
pub struct JsVariable {
    pub name: String,
    pub kind: usize,
    pub size: usize,
    pub dimensions: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub length: usize,
}

#[derive(Serialize, Deserialize)]
pub struct JsDimension {
    pub name: String,
    pub length: usize,
}

#[wasm_bindgen]
pub struct NetCDFHandle {
    file: NetCDF<WasmFile>,
}

pub fn new_wrapper(file: NetCDF<WasmFile>) -> NetCDFHandle {
    NetCDFHandle { file }
}

#[wasm_bindgen]
impl NetCDFHandle {
    #[wasm_bindgen]
    pub fn get_map_size(&self) -> Result<usize, JsValue> {
        self.file.mapsize().map_err(|err| err.into_js())
    }

    #[wasm_bindgen]
    pub fn get_variables(&self) -> Result<Vec<JsValue>, JsValue> {
        let header = self.file.header();
        let vars = match &header.vars {
            Some(vars) => vars,
            None => {
                let err: HeadScratcherError<String> = HeadScratcherError::NoVariablesInFile;
                return Err(err.into_js())
            },
        };

        let js_vars = vars
            .iter()
            .map(|(key, var)| {
                let kind = match var.nc_type {
                    NetCDFType::NC_BYTE => 1,
                    NetCDFType::NC_CHAR => 2,
                    NetCDFType::NC_SHORT => 3,
                    NetCDFType::NC_INT => 4,
                    NetCDFType::NC_FLOAT => 5,
                    NetCDFType::NC_DOUBLE => 6,
                } as usize;

                let attributes: HashMap<String, String> = var
                    .attributes()
                    .as_ref()
                    .and_then(|attrs| {
                        attrs
                            .iter()
                            .filter_map(|(key, value)| {
                                value.as_string().map(|str| Some((key.clone(), str)))
                            })
                            .collect()
                    })
                    .unwrap_or(HashMap::new());

                let temp_dim_map = HashMap::new();
                let dims = header.dims.as_ref().unwrap_or(&temp_dim_map);
                let var_dims = var
                    .dims
                    .iter()
                    .map(|dim_index| {
                        let index = *dim_index as usize;
                        dims.get(&index).unwrap().name()
                    })
                    .collect();

                JsVariable {
                    name: key.clone(),
                    kind,
                    attributes,
                    size: var.nc_type.extsize(),
                    length: var.length(),
                    dimensions: var_dims,
                }
            })
            .map(|var| JsValue::from_serde(&var).unwrap())
            .collect();
        Ok(js_vars)
    }

    #[wasm_bindgen]
    pub fn get_dimensions(&self) -> Vec<JsValue> {
        let header = self.file.header();
        match &header.dims {
            Some(dims) => dims
                .iter()
                .map(|(_i, dim)| JsDimension {
                    name: dim.name(),
                    length: dim.length,
                })
                .map(|dim| JsValue::from_serde(&dim).unwrap())
                .collect(),
            None => return Vec::new(),
        }
    }

    #[wasm_bindgen]
    pub fn get_variable_size(&self, variable: String) -> usize {
        let header = self.file.header();
        header
            .vars
            .as_ref()
            .and_then(|vars| vars.get(&variable))
            .map(|var| var.nc_type.extsize())
            .unwrap_or(0)
    }

    #[wasm_bindgen]
    pub fn get_attribute(&self, name: String) -> JsValue {
        let header = self.file.header();
        header
            .attrs
            .as_ref()
            .and_then(|attrs| attrs.get(&name))
            .and_then(|attr| attr.as_string())
            .map(|str| JsValue::from(str))
            .unwrap_or(JsValue::null())
    }

    #[wasm_bindgen]
    pub fn load_data_for(
        &mut self,
        variable: String,
        index: &mut [usize],
        data: &mut [u8],
    ) -> Result<(), JsValue> {
        self.file
            .update_buffer(variable, index, data)
            .map_err(|err| err.into_js())
    }
}
