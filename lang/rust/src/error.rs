//! Crate errors.
use std::result::Result as StdResult;

#[derive(Debug)]
pub enum Error {
    Other(String),

    /// A runtime error.
    Runtime(String),

    /// A value error.
    Value(String),
}

pub type Result<T = ()> = StdResult<T, Error>;
