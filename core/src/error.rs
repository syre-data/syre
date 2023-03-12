//! Common error types.
use crate::db::error::Error as DbError;
use crate::types::{ResourceId, ResourcePath};
use std::convert::From;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::result::Result as StdResult;

#[cfg(feature = "serde")]
use serde::{self, Deserialize, Serialize};

// **********************
// *** Resource Error ***
// **********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum ResourceError {
    DoesNotExist(&'static str),
    DuplicateId(ResourceId),
    AlreadyExists(&'static str),
}

// **********************
// *** Project Error ***
// **********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum ProjectError {
    NotRegistered(Option<ResourceId>, Option<PathBuf>),
    Misconfigured(&'static str),
}

// ******************
// *** GraphError ***
// ******************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum GraphError {
    InvalidGraph(&'static str),
}

// ***********************
// *** Container Error ***
// ***********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum ContainerError {
    MissingChild(ResourceId),
}

// *******************
// *** Asset Error ***
// *******************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum AssetError {
    NotRegistered(Option<ResourceId>, Option<ResourcePath>),
    PathNotSet,
}

// ********************
// *** Script Error ***
// ********************

// @todo: serde features.
#[derive(Debug)]
pub enum ScriptError {
    UnknownLanguage(Option<OsString>),
}

// ***************************
// *** Resource Path Error ***
// ***************************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum ResourcePathError {
    CouldNotParseMetalevel(&'static str),
}

// ********************
// *** Runner Error ***
// ********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum RunnerError {
    /// An error occured when running the script
    /// on the specified `Container`.
    ///
    /// # Fields
    /// 1. [`ResourceId`] of the `Script`.
    /// 2. [`ResourceId`] of the `Container`.
    /// 3. Error message from the script.
    ScriptError(ResourceId, ResourceId, String),
}

// ******************
// *** Thot Error ***
// ******************

// @todo[3]: Put behind correct features.
#[derive(Debug)]
pub enum Error {
    AssetError(AssetError),
    ContainerError(ContainerError),
    DbError(DbError),
    IoError(io::Error),
    ProjectError(ProjectError),
    ResourceError(ResourceError),
    GraphError(GraphError),
    ResourcePathError(ResourcePathError),
    RunnerError(RunnerError),
    ScriptError(ScriptError),
    SerdeError(serde_json::Error),

    /// Invalid value encountered.
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
