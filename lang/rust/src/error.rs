//! Crate errors.
use std::result::Result as StdResult;
use syre_local_database::Error as DbError;

#[derive(Debug)]
pub enum Error {
    Other(String),

    /// A runtime error.
    Runtime(String),

    /// A value error.
    Value(String),

    /// `syre_local_database` error.
    Database(DbError),

    ZMQ(zmq::Error),
}

impl From<DbError> for Error {
    fn from(err: DbError) -> Self {
        Self::Database(err)
    }
}

impl From<zmq::Error> for Error {
    fn from(value: zmq::Error) -> Self {
        Self::ZMQ(value)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
