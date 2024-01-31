//! Common error types.
use crate::types::ResourceId;
use std::collections::HashMap;
use std::convert::From;
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
pub enum Resource {
    #[error("resource `{0}` does not exist")]
    DoesNotExist(String),

    #[error("id `{0}` already exists")]
    DuplicateId(ResourceId),

    #[error("resource `{0}` already exists")]
    AlreadyExists(String),
}

impl Resource {
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
pub enum Graph {
    #[error("invalid graph: {0}")]
    InvalidGraph(String),

    #[error("illegal operation: {0}")]
    IllegalOperation(String),
}

impl Graph {
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
    NotRegistered(Option<ResourceId>, Option<PathBuf>),

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
// *** Syre Error ***
// ******************

// TODO Put behind correct features.
#[derive(Error, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Error {
    #[error("{0}")]
    AssetError(AssetError),

    #[error("{0}")]
    Project(Project),

    #[error("{0}")]
    Resource(Resource),

    #[error("{0}")]
    Graph(Graph),

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

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeError(err)
    }
}

impl From<Resource> for Error {
    fn from(err: Resource) -> Self {
        Self::Resource(err)
    }
}

impl From<Runner> for Error {
    fn from(err: Runner) -> Self {
        Self::Runner(err)
    }
}

impl From<Graph> for Error {
    fn from(err: Graph) -> Self {
        Self::Graph(err)
    }
}

impl From<ScriptError> for Error {
    fn from(err: ScriptError) -> Self {
        Self::ScriptError(err)
    }
}

// *******************
// *** Syre Result ***
// *******************

pub type Result<T = ()> = StdResult<T, Error>;

impl From<Error> for Result {
    fn from(err: Error) -> Self {
        Err(err)
    }
}
