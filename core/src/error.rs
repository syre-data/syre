//! Common error types.
use crate::types::{ResourceId, ResourcePath};
use std::convert::From;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::result::Result as StdResult;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{self, Deserialize, Serialize};

// **********************
// *** Resource Error ***
// **********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("resource `{0}` does not exist")]
    DoesNotExist(&'static str),

    #[error("id `{0}` already exists")]
    DuplicateId(ResourceId),

    #[error("resource `{0}` already exists")]
    AlreadyExists(&'static str),
}

// **********************
// *** Project Error ***
// **********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Project is not registered")]
    NotRegistered(Option<ResourceId>, Option<PathBuf>),

    #[error("Project is misconfigured: {0}")]
    Misconfigured(&'static str),
}

// ******************
// *** GraphError ***
// ******************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum GraphError {
    #[error("invalid graph: {0}")]
    InvalidGraph(&'static str),
}

// *******************
// *** Asset Error ***
// *******************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not registered")]
    NotRegistered(Option<ResourceId>, Option<ResourcePath>),

    #[error("Asset path is not set")]
    PathNotSet,
}

// ********************
// *** Script Error ***
// ********************

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("unknown language `{0:?}`")]
    UnknownLanguage(Option<OsString>),
}

// ***************************
// *** Resource Path Error ***
// ***************************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum ResourcePathError {
    #[error("{0}")]
    CouldNotParseMetalevel(&'static str),
}

// ********************
// *** Runner Error ***
// ********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum RunnerError {
    /// An error occured when running the script
    /// on the specified `Container`.
    ///
    /// # Fields
    /// 1. [`ResourceId`] of the `Script`.
    /// 2. [`ResourceId`] of the `Container`.
    /// 3. Error message from the script.
    #[error("Script `{0}` running over Container `{1}` errored: {2}")]
    ScriptError(ResourceId, ResourceId, String),
}

// ******************
// *** Thot Error ***
// ******************

// TODO Put behind correct features.
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    AssetError(AssetError),

    #[error("{0}")]
    IoError(io::Error),

    #[error("{0}")]
    ProjectError(ProjectError),

    #[error("{0}")]
    ResourceError(ResourceError),

    #[error("{0}")]
    GraphError(GraphError),

    #[error("{0}")]
    ResourcePathError(ResourcePathError),

    #[error("{0}")]
    RunnerError(RunnerError),

    #[error("{0}")]
    ScriptError(ScriptError),

    #[error("{0}")]
    SerdeError(serde_json::Error),

    /// Invalid value encountered.
    #[error("{0}")]
    ValueError(&'static str),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeError(err)
    }
}

impl From<ResourceError> for Error {
    fn from(err: ResourceError) -> Self {
        Self::ResourceError(err)
    }
}

impl From<RunnerError> for Error {
    fn from(err: RunnerError) -> Self {
        Self::RunnerError(err)
    }
}

impl From<GraphError> for Error {
    fn from(err: GraphError) -> Self {
        Self::GraphError(err)
    }
}

// @todo: Make better.
#[cfg(feature = "serde")]
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let msg = format!("{:?}", self);
        serializer.serialize_str(msg.as_ref())
    }
}

// *******************
// *** Thot Result ***
// *******************

pub type Result<T = ()> = StdResult<T, Error>;

impl From<Error> for Result {
    fn from(err: Error) -> Self {
        Err(err)
    }
}

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
