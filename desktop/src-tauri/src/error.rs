//! `Error`s and `Result`s
use serde::{Deserialize, Serialize};
use std::io;
use std::result::Result as StdResult;
use syre_core::Error as CoreError;
use syre_local::Error as LocalError;
use syre_local_database::error::Error as DbError;
use tauri::Error as TauriError;
use thiserror::Error;

// ******************************
// *** Desktop Settings Error ***
// ******************************

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum DesktopSettings {
    /// The desired update can not be performed.
    #[error("{0}")]
    InvalidUpdate(String),

    /// An active user is not set.
    #[error("no user activated")]
    NoUser,
}

// *************
// *** Error ***
// *************

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    CoreError(CoreError),

    #[error("{0}")]
    DesktopSettings(DesktopSettings),

    #[error("{0}")]
    SerdeError(serde_json::Error),

    #[error("{0}")]
    IoError(io::Error),

    #[error("{0}")]
    LocalError(LocalError),

    #[error("{0}")]
    TauriError(TauriError),

    #[error("{0}")]
    LocalDatabaseError(DbError),

    #[error("{0}")]
    IoSerde(syre_local::error::IoSerde),
}

impl From<DesktopSettings> for Error {
    fn from(err: DesktopSettings) -> Self {
        Self::DesktopSettings(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Self::CoreError(err)
    }
}

impl From<LocalError> for Error {
    fn from(err: LocalError) -> Self {
        Self::LocalError(err)
    }
}

impl From<DbError> for Error {
    fn from(err: DbError) -> Self {
        Self::LocalDatabaseError(err)
    }
}

impl From<TauriError> for Error {
    fn from(err: TauriError) -> Self {
        Self::TauriError(err)
    }
}

impl From<syre_local::error::IoSerde> for Error {
    fn from(value: syre_local::error::IoSerde) -> Self {
        Self::IoSerde(value)
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// **************
// *** Result ***
// **************

pub type Result<T = ()> = StdResult<T, Error>;
