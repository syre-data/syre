//! Errors
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use thiserror::Error;
use thot_core::Error as CoreError;
use thot_local::Error as LocalError;

#[cfg(feature = "server")]
use crate::types::SocketType;

// **************
// ***  Error ***
// **************

/// [`Database`](crate::db) related errors.
#[derive(Serialize, Deserialize, Error, Debug)]
pub enum Error {
    /// A ZMQ [`Context`](zmq::Context) does not exist where expected.
    #[error("ZMQ context does not exist")]
    ContextDoesNotExist,

    /// A ZMQ error.
    #[error("{0}")]
    ZMQ(String),

    /// A type of socket is required, but has not yet been created.
    #[cfg(feature = "server")]
    #[error("{0:?}")]
    SocketDoesNotExist(SocketType),

    #[cfg(feature = "server")]
    #[error("{0}")]
    SettingsError(String),

    #[error("{0}")]
    CoreError(String),

    #[error("{0}")]
    LocalError(String),

    /// Issue with the database.
    #[error("{0}")]
    DatabaseError(String),
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

#[cfg(feature = "server")]
impl From<trash::Error> for Error {
    fn from(err: trash::Error) -> Self {
        Error::LocalError(format!("{err:?}"))
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
