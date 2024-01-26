//! Common error types.
use crate::types::{ResourceId, ResourcePath};
use std::collections::HashMap;
use std::convert::From;
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
    DoesNotExist(String),

    #[error("id `{0}` already exists")]
    DuplicateId(ResourceId),

    #[error("resource `{0}` already exists")]
    AlreadyExists(String),
}

impl ResourceError {
    pub fn does_not_exist(msg: impl Into<String>) -> Self {
        Self::DoesNotExist(msg.into())
    }

    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }
}

// **********************
// *** Project Error ***
// **********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Clone, Debug)]
pub enum Project {
    #[error("Project is not registered")]
    NotRegistered(Option<ResourceId>, Option<PathBuf>),

    #[error("Project is misconfigured: {0}")]
    Misconfigured(String),
}

impl Project {
    pub fn misconfigured(msg: impl Into<String>) -> Self {
        Self::Misconfigured(msg.into())
    }
}

// ******************
// *** GraphError ***
// ******************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum GraphError {
    #[error("invalid graph: {0}")]
    InvalidGraph(String),
}

impl GraphError {
    pub fn invalid_graph(msg: impl Into<String>) -> Self {
        Self::InvalidGraph(msg.into())
    }
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ScriptError {
    #[error("unknown language `{0:?}`")]
    UnknownLanguage(Option<String>),
}

// ***************************
// *** Resource Path Error ***
// ***************************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum ResourcePathError {
    #[error("{0}")]
    CouldNotParseMetalevel(String),
}

impl ResourcePathError {
    pub fn could_not_parse_meta_level(msg: impl Into<String>) -> Self {
        Self::CouldNotParseMetalevel(msg.into())
    }
}

// ********************
// *** Runner Error ***
// ********************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Error, Debug)]
pub enum Runner {
    #[error("{0:?}")]
    LoadScripts(HashMap<ResourceId, String>),

    /// The `Container` could not be found in the graph.
    #[error("Container {0} not found")]
    ContainerNotFound(ResourceId),

    /// An error occured when running the script
    /// on the specified `Container`.
    #[error("Script `{script}` running over Container `{container}` errored: {description}")]
    ScriptError {
        script: ResourceId,
        container: ResourceId,
        description: String,
    },

    #[error("error running `{cmd}` from script `{script}` on container `{container}`")]
    CommandError {
        script: ResourceId,
        container: ResourceId,
        cmd: String,
    },
}

// ******************
// *** Thot Error ***
// ******************

// TODO Put behind correct features.
#[derive(Error, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Error {
    #[error("{0}")]
    AssetError(AssetError),

    #[error("{0}")]
    #[cfg_attr(feature = "serde", serde(skip))]
    IoError(io::Error),

    #[error("{0}")]
    Project(Project),

    #[error("{0}")]
    ResourceError(ResourceError),

    #[error("{0}")]
    GraphError(GraphError),

    #[error("{0}")]
    ResourcePathError(ResourcePathError),

    #[error("{0}")]
    Runner(Runner),

    #[error("{0}")]
    ScriptError(ScriptError),

    #[error("{0}")]
    #[cfg_attr(feature = "serde", serde(skip))]
    SerdeError(serde_json::Error),

    /// Invalid value encountered.
    #[error("{0}")]
    Value(String),
}

impl Error {
    pub fn value(msg: impl Into<String>) -> Self {
        Self::Value(msg.into())
    }
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

impl From<Runner> for Error {
    fn from(err: Runner) -> Self {
        Self::Runner(err)
    }
}

impl From<GraphError> for Error {
    fn from(err: GraphError) -> Self {
        Self::GraphError(err)
    }
}

impl From<ScriptError> for Error {
    fn from(err: ScriptError) -> Self {
        Self::ScriptError(err)
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
