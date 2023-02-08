//! Errors and Result.
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::path::PathBuf;
use std::result::Result as StdResult;

// **************
// *** Errors ***
// **************

/// Used for errors specifically related to settings.
#[derive(Serialize, Deserialize, Debug)]
pub enum SettingsError {
    InvalidPath(PathBuf),

    /// A required path has not yet been set.
    PathNotSet,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    SerdeError(serde_json::Error),
    SettingsError(SettingsError),
}

// ***************
// *** Results ***
// ***************

pub type Result<T = ()> = StdResult<T, Error>;

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
