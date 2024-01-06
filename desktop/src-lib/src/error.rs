//! Errors
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use thot_core::error::Error as CoreError;

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
    Core(CoreError),
}

impl From<DesktopSettings> for Error {
    fn from(err: DesktopSettings) -> Self {
        Self::DesktopSettings(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
