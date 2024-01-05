//! `Error`s and `Result`s
use serde::{Deserialize, Serialize};
use std::io;
use std::result::Result as StdResult;
use tauri::Error as TauriError;
use thiserror::Error;
use thot_core::Error as CoreError;
use thot_local::Error as LocalError;
use thot_local_database::error::Error as DbError;

// ******************************
// *** Desktop Settings Error ***
// ******************************

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum DesktopSettingsError {
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
    DesktopSettingsError(DesktopSettingsError),

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
    Save(thot_local::error::Save),
}

impl From<DesktopSettingsError> for Error {
    fn from(err: DesktopSettingsError) -> Self {
        Self::DesktopSettingsError(err)
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

impl From<thot_local::error::Save> for Error {
    fn from(value: thot_local::error::Save) -> Self {
        Self::Save(value)
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
