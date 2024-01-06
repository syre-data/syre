//! Errors
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::result::Result as StdResult;
use thiserror::Error;
use thot_core::Error as CoreError;
use thot_local::error::{Error as LocalError, IoSerde};

#[cfg(feature = "server")]
use crate::types::SocketType;

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

    #[error("{0:?}")]
    CoreError(CoreError),

    #[error("{0}")]
    LocalError(LocalError),

    #[error("{0}")]
    TrashError(String),

    /// Issue with the database.
    #[error("{0}")]
    DatabaseError(String),

    /// The database has become out of sync.
    #[error("database out of sync")]
    OutOfSync,

    #[error("{0}")]
    IoSerde(IoSerde),

    #[error("{0}")]
    LoadContainer(thot_local::loader::error::container::Error),

    #[error("{0:?}")]
    LoadTree(HashMap<PathBuf, thot_local::loader::error::tree::Error>),

    #[error("{errors:?}")]
    LoadPartial {
        errors: HashMap<PathBuf, thot_local::loader::error::tree::Error>,
        graph: Option<thot_core::graph::ResourceTree<thot_core::project::Container>>,
    },
}

#[cfg(any(feature = "server", feature = "client"))]
impl From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Self {
        Self::ZMQ(err.message().to_string())
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

impl From<IoSerde> for Error {
    fn from(value: IoSerde) -> Self {
        Self::IoSerde(value)
    }
}

#[cfg(feature = "server")]
impl From<thot_local::loader::error::container::Error> for Error {
    fn from(value: thot_local::loader::error::container::Error) -> Self {
        Self::LoadContainer(value)
    }
}

#[cfg(feature = "server")]
impl From<HashMap<PathBuf, thot_local::loader::error::tree::Error>> for Error {
    fn from(value: HashMap<PathBuf, thot_local::loader::error::tree::Error>) -> Self {
        Self::LoadTree(value)
    }
}

#[cfg(feature = "server")]
impl From<trash::Error> for Error {
    fn from(err: trash::Error) -> Self {
        Error::TrashError(format!("{err:?}"))
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
