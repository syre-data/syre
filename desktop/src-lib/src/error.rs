//! Errors
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    DatabaseError(String),
}

pub type Result<T = ()> = StdResult<T, Error>;

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
