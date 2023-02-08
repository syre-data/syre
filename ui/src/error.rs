//! Errors and results.
use std::result::Result as StdResult;
use wasm_bindgen::JsValue;

// *************
// *** Error ***
// *************

pub enum Error {
    JsValueError(JsValue),
}

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        Error::JsValueError(err)
    }
}

// **************
// *** Result ***
// **************

pub type Result<T = ()> = StdResult<T, Error>;

#[cfg(test)]
#[path = "./result_test.rs"]
mod result_test;
