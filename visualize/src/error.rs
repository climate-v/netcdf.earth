use headscratcher::error::HeadScratcherError;
use serde::Serialize;
use wasm_bindgen::JsValue;

// This is not great, but it's probably the best solution.
// The big issue with doing this in head-scratcher directly is that
// some types that are used are not serializable, specifically `nom`
// errors and IO errors. Generally we should not encounter
// nom-errors on our side, but the type still contains these and
// thus would have to provide a way to serialize them.
/// Matching definition for [HeadScratcherError] so that it's serializable.
/// Notably however, some entries had to be adjusted to allow them to be
/// serialized and thus differ.
#[derive(Serialize)]
pub enum HeadScratcherErrorDef {
    EmptyError,
    InvalidFile,
    UnsupportedNetCDFVersion,
    UnsupportedListType(u32),
    NonZeroValue(u32),
    UnsupportedZeroListType,
    UTF8error,
    UnknownNetCDFType(usize),
    ParsingError,
    IOError(String),
    NoVariablesInFile,
    NoDimensionsInFile,
    VariableNotFound(String),
    CouldNotFindDimension(String),
}

impl<I> From<HeadScratcherError<I>> for HeadScratcherErrorDef {
    fn from(err: HeadScratcherError<I>) -> Self {
        return match err {
            HeadScratcherError::EmptyError => HeadScratcherErrorDef::EmptyError,
            HeadScratcherError::UnsupportedNetCDFVersion => {
                HeadScratcherErrorDef::UnsupportedNetCDFVersion
            }
            HeadScratcherError::UnsupportedListType(list) => {
                HeadScratcherErrorDef::UnsupportedListType(list)
            }
            HeadScratcherError::NonZeroValue(val) => HeadScratcherErrorDef::NonZeroValue(val),
            HeadScratcherError::UnsupportedZeroListType => {
                HeadScratcherErrorDef::UnsupportedZeroListType
            }
            HeadScratcherError::UTF8error => HeadScratcherErrorDef::UTF8error,
            HeadScratcherError::UnknownNetCDFType(tpe) => {
                HeadScratcherErrorDef::UnknownNetCDFType(tpe)
            }
            HeadScratcherError::NomError(_,_) => HeadScratcherErrorDef::ParsingError,
            HeadScratcherError::IOError(io) => {
                let err: std::io::Error = io.into();
                HeadScratcherErrorDef::IOError(err.to_string())
            },
            HeadScratcherError::NoVariablesInFile => HeadScratcherErrorDef::NoVariablesInFile,
            HeadScratcherError::NoDimensionsInFile => HeadScratcherErrorDef::NoDimensionsInFile,
            HeadScratcherError::VariableNotFound(var) => {
                HeadScratcherErrorDef::VariableNotFound(var)
            }
            HeadScratcherError::CouldNotFindDimension(dim) => {
                HeadScratcherErrorDef::CouldNotFindDimension(dim)
            }
            HeadScratcherError::InvalidFile => HeadScratcherErrorDef::InvalidFile,
        };
    }
}

pub trait ToJs {
    fn into_js(self) -> JsValue;
}

impl<I> ToJs for HeadScratcherError<I> {
    fn into_js(self) -> JsValue {
        let my_error: HeadScratcherErrorDef = self.into();
        JsValue::from_serde(&my_error).unwrap()
    }
}
