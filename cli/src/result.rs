use clap::error::Error as ClapError;
use settings_manager::result::Error as SettingsError;
use std::io;
use std::result::Result as StdResult;
use thot_core::result::Error as CoreError;
use thot_local::result::Error as LocalError;

// *************
// *** Error ***
// *************

#[derive(Debug)]
pub enum Error {
    ClapError(ClapError),
    CoreError(CoreError),
    IoError(io::Error),
    LocalError(LocalError),
    SettingsError(SettingsError),
}

impl From<ClapError> for Error {
    fn from(err: ClapError) -> Self {
        Error::ClapError(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::CoreError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<LocalError> for Error {
    fn from(err: LocalError) -> Self {
        Error::LocalError(err)
    }
}

impl From<SettingsError> for Error {
    fn from(err: SettingsError) -> Self {
        Error::SettingsError(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;

#[cfg(test)]
#[path = "./result_test.rs"]
mod result_test;
