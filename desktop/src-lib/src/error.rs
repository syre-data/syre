//! Errors
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;

// ******************************
// *** Desktop Settings Error ***
// ******************************

#[derive(Serialize, Deserialize, Debug)]
pub enum DesktopSettings {
    /// The desired update can not be performed.
    InvalidUpdate(String),

    /// An active user is not set.
    NoUser,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    DesktopSettings(DesktopSettings),
    Database(String),
}

impl From<DesktopSettings> for Error {
    fn from(err: DesktopSettings) -> Self {
        Self::DesktopSettings(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
