//! Common error types.
use serde::{self, Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use std::result::Result as StdResult;
use thot_core::types::ResourceId;
use thot_core::Error as CoreError;

// ***********************
// *** Settings Errors ***
// ***********************

#[cfg(feature = "fs")]
#[derive(Debug)]
pub enum SettingsFileError {
    CouldNotLoad(PathBuf),
    CouldNotSave(PathBuf),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SettingsValidationError {
    InvalidSetting,
}

// **********************
// *** Project Error ***
// **********************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Debug)]
pub enum ProjectError {
    DuplicatePath(PathBuf),
    PathNotAProjectRoot(PathBuf),
    PathNotInProject(PathBuf),
    PathNotAResource(PathBuf),
}

// ***********************
// *** Container Error ***
// ***********************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Debug)]
pub enum ContainerError {
    InvalidChildPath(PathBuf),

    /// If a path is expected to represent a [`Container`](crate::project::resources::Container)
    /// but does not.
    PathNotAContainer(PathBuf),
}

// *******************
// *** Asset Error ***
// *******************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Debug)]
pub enum AssetError {
    PathNotAContainer(PathBuf),
    FileAlreadyAsset(PathBuf),
    ContainerNotFound(PathBuf),
    InvalidPath(PathBuf, String),

    /// The [`AssetFileAction`](crate::types::AssetFileAction) is
    /// incompatible with the path.
    IncompatibleAction(String),

    /// An error occured in the process of using the
    /// [`AssetBuilder`](crate::project::asset::AssetBuilder).
    BuilderError(String),
}

// ********************
// *** Users Errors ***
// ********************

#[derive(Serialize, Deserialize, Debug)]
pub enum UsersError {
    DuplicateEmail(String),
    InvalidEmail(String),
}

// ****************************
// *** Resource Store Error ***
// ****************************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Debug)]
pub enum ResourceStoreError {
    /// If a [`ResourceId`] is expected to be present as a map key, but is not.
    IdNotPresent(ResourceId),

    /// If trying to get an empty value.
    LoadEmptyValue,

    /// If trying to set a value but the resource is already loaded.
    ResourceAlreadyLoaded,
}

// *******************
// *** Local Error ***
// *******************

#[derive(Debug)]
pub enum Error {
    CoreError(CoreError),
    InvalidPath(PathBuf),
    SettingsValidationError(SettingsValidationError),
    UsersError(UsersError),

    #[cfg(feature = "fs")]
    AssetError(AssetError),

    #[cfg(feature = "fs")]
    ContainerError(ContainerError),

    #[cfg(feature = "fs")]
    ProjectError(ProjectError),

    #[cfg(feature = "fs")]
    ResourceStoreError(ResourceStoreError),

    #[cfg(feature = "fs")]
    SettingsFileError(SettingsFileError),
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::CoreError(err)
    }
}

#[cfg(feature = "fs")]
impl From<ContainerError> for Error {
    fn from(err: ContainerError) -> Self {
        Error::ContainerError(err)
    }
}

#[cfg(feature = "fs")]
impl From<AssetError> for Error {
    fn from(err: AssetError) -> Self {
        Error::AssetError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::CoreError(CoreError::IoError(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::CoreError(CoreError::SerdeError(err))
    }
}

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
