use clap::error::Error as ClapError;
use std::io;
use std::result::Result as StdResult;
use thot_core::Error as CoreError;
use thot_local::Error as LocalError;

// *************
// *** Error ***
// *************

#[derive(Debug)]
pub enum Error {
    Clap(ClapError),
    Core(CoreError),
    Io(io::Error),
    Local(LocalError),
}

impl From<ClapError> for Error {
    fn from(err: ClapError) -> Self {
        Error::Clap(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::Core(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<LocalError> for Error {
    fn from(err: LocalError) -> Self {
        Error::Local(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;

#[cfg(test)]
#[path = "./result_test.rs"]
mod result_test;
