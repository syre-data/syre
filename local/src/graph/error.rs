use crate::error::{IoErrorKind, LoadError};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum LoaderError {
    #[error("{0}")]
    Load(LoadError),

    #[error("{kind}: {path}")]
    Io {
        path: PathBuf,

        #[serde(with = "IoErrorKind")]
        kind: io::ErrorKind,
    },
}

impl From<LoadError> for LoaderError {
    fn from(err: LoadError) -> Self {
        Self::Load(err)
    }
}
