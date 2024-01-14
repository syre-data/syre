//! Errors
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use thiserror::Error;
use thot_core::error::Error as CoreError;

#[derive(Serialize, Deserialize, thiserror::Error, Debug)]
pub enum Trash {
    /// File was not found.
    #[error{"not found"}]
    NotFound,

    #[error("{0}")]
    Other(String),
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum RemoveAsset {
    #[error("{0}")]
    ZMQ(String),

    #[error("{0}")]
    Trash(Trash),

    #[error("{0}")]
    Database(String),
}

impl From<Trash> for RemoveAsset {
    fn from(value: Trash) -> Self {
        Self::Trash(value)
    }
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum Analysis {
    #[error("{0}")]
    ZMQ(String),

    #[error("could not find graph")]
    GraphNotFound,

    #[error("{0}")]
    Analysis(thot_core::error::Runner),
}

impl From<thot_core::error::Runner> for Analysis {
    fn from(value: thot_core::error::Runner) -> Self {
        Self::Analysis(value)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DesktopSettings {
    /// The desired update can not be performed.
    InvalidUpdate(String),

    /// An active user is not set.
    NoUser,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    DesktopSettings(DesktopSettings),
    Core(CoreError),
}

impl From<DesktopSettings> for Error {
    fn from(err: DesktopSettings) -> Self {
        Self::DesktopSettings(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;
