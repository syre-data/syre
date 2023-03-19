//! `Error`s and `Result`s
use settings_manager::Error as SettingsError;
use std::error::Error as StdErrorTrait;
use std::result::Result as StdResult;
use std::{fmt, io};
use tauri::Error as TauriError;
use thot_core::Error as CoreError;
use thot_local::Error as LocalError;
use thot_local_database::error::Error as DbError;

// ******************************
// *** Desktop Settings Error ***
// ******************************

#[derive(Debug)]
pub enum DesktopSettingsError {
    /// The desired update can not be performed.
    InvalidUpdate(String),

    /// An active user is not set.
    NoUser,
}

// *************
// *** Error ***
// *************

#[derive(Debug)]
pub enum Error {
    CoreError(CoreError),
    DesktopSettingsError(DesktopSettingsError),
    IoError(io::Error),
    LocalError(LocalError),
    SettingsError(SettingsError),
    TauriError(TauriError),
    LocalDatabaseError(DbError),
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

impl From<SettingsError> for Error {
    fn from(err: SettingsError) -> Self {
        Self::SettingsError(err)
    }
}

impl From<TauriError> for Error {
    fn from(err: TauriError) -> Self {
        Self::TauriError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl StdErrorTrait for Error {}

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

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
