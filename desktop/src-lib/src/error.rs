//! Errors
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use thot_core::error::Error as CoreError;
use thot_local_database::error::Error as DbError;

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
    Database(DbError),
    Core(CoreError),
}

impl From<DesktopSettings> for Error {
    fn from(err: DesktopSettings) -> Self {
        Self::DesktopSettings(err)
    }
}

impl From<DbError> for Error {
    fn from(err: DbError) -> Self {
        Self::Database(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
