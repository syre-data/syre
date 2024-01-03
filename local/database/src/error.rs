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

    #[error("core error")]
    CoreError(CoreError),

    #[error("{0}")]
    LocalError(LocalError),

    #[error("{0}")]
    TrashError(String),

    /// Issue with the database.
    #[error("{0}")]
    DatabaseError(String),

    /// The database has become out of sync.
    #[error("out of sync")]
    OutOfSync,
}

impl From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Self {
        Error::ZMQ(err.message().to_string())
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::CoreError(err)
    }
}

impl From<LocalError> for Error {
    fn from(err: LocalError) -> Self {
        Error::LocalError(err)
    }
}

impl From<thot_local::error::LoadError> for Error {
    fn from(err: thot_local::error::LoadError) -> Self {
        Error::LocalError(err.into())
    }
}

impl From<thot_local::error::LoaderErrors> for Error {
    fn from(err: thot_local::error::LoaderErrors) -> Self {
        Error::LocalError(err.into())
    }
}

#[cfg(feature = "server")]
impl From<trash::Error> for Error {
    fn from(err: trash::Error) -> Self {
        Error::TrashError(format!("{err:?}"))
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
