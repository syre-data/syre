//! Crate errors.
use std::result::Result as StdResult;
use thot_local_database::Error as DbError;

#[derive(Debug)]
pub enum Error {
    Other(String),

    /// A runtime error.
    Runtime(String),

    /// A value error.
    Value(String),

    /// `thot_local_database` error.
    Database(DbError),
}

impl From<DbError> for Error {
    fn from(err: DbError) -> Self {
        Self::Database(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
