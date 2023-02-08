//! Errors
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use thot_core::Error as CoreError;
use thot_local::Error as LocalError;

#[cfg(feature = "server")]
use crate::types::SocketType;

#[cfg(feature = "server")]
use settings_manager::Error as SettingsError;

// **************
// ***  Error ***
// **************

/// [`Database`](crate::db) related errors.
#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    /// A ZMQ [`Context`](zmq::Context) does not exist where expected.
    ContextDoesNotExist,

    // @todo: Serialize using `message`.
    /// A ZMQ error.
    ZMQ(String),

    /// A type of socket is required, but has not yet been created.
    #[cfg(feature = "server")]
    SocketDoesNotExist(SocketType),

    // @todo: Serialize correctly.
    #[cfg(feature = "server")]
    SettingsError(String),

    // @todo: Serialize correctly.
    CoreError(String),

    // @todo: Serialize correctly.
    LocalError(String),
}

#[cfg(feature = "server")]
impl From<SettingsError> for Error {
    fn from(err: SettingsError) -> Self {
        // @todo: Serialize correctly.
        Error::SettingsError(format!("{err:?}"))
    }
}

impl From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Self {
        Error::ZMQ(err.message().to_string())
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::CoreError(format!("{err:?}"))
    }
}

impl From<LocalError> for Error {
    fn from(err: LocalError) -> Self {
        Error::LocalError(format!("{err:?}"))
    }
}

pub type Result<T = ()> = StdResult<T, Error>;

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
