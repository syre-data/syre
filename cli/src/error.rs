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
    LoadContainer(thot_local::loader::error::container::Error),
    LoadTree(thot_local::loader::error::tree::Error),
    IoSerde(thot_local::error::IoSerde),
}

impl From<ClapError> for Error {
    fn from(err: ClapError) -> Self {
        Self::Clap(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<LocalError> for Error {
    fn from(err: LocalError) -> Self {
        Self::Local(err)
    }
}

impl From<thot_local::loader::error::container::Error> for Error {
    fn from(value: thot_local::loader::error::container::Error) -> Self {
        Self::LoadContainer(value)
    }
}

impl From<thot_local::loader::error::tree::Error> for Error {
    fn from(value: thot_local::loader::error::tree::Error) -> Self {
        Self::LoadTree(value)
    }
}

impl From<thot_local::error::IoSerde> for Error {
    fn from(value: thot_local::error::IoSerde) -> Self {
        Self::IoSerde(value)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
