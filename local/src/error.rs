//! Common error types.
use crate::graph::error::LoaderError;
use serde::{self, Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use std::result::Result as StdResult;
use thiserror::Error;
use thot_core::Error as CoreError;

// ***********************
// *** Settings Errors ***
// ***********************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Error, Debug)]
pub enum SettingsFileError {
    #[error("could not load `{0}`")]
    CouldNotLoad(PathBuf),

    #[error("could not save `{0}`")]
    CouldNotSave(PathBuf),
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum SettingsValidationError {
    #[error("invalid settings")]
    InvalidSetting,
}

// ******************
// *** Load Error ***
// ******************

#[cfg(feature = "fs")]
pub type LoaderErrors = Vec<LoaderError>;

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Error, Debug)]
pub enum LoadError {
    #[error("`{0}` does not exist")]
    DoesNotExist(PathBuf),

    #[error("could not open `{0}`")]
    CouldNotOpen(PathBuf),

    #[error("invalid json `{0}`")]
    InvalidJson(PathBuf),
}

// **********************
// *** Project Error ***
// **********************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Error, Debug)]
pub enum ProjectError {
    #[error("`{0}` already registered")]
    DuplicatePath(PathBuf),

    #[error("`{0}` not a Project root")]
    PathNotAProjectRoot(PathBuf),

    #[error("`{0}` not in a Project")]
    PathNotInProject(PathBuf),

    #[error("`{0}` is not a resource")]
    PathNotAResource(PathBuf),

    #[error("`{0}` is not registered")]
    PathNotRegistered(PathBuf),
}

// ***********************
// *** Container Error ***
// ***********************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Error, Debug)]
pub enum ContainerError {
    #[error("`{0}` is not a child Container")]
    InvalidChildPath(PathBuf),

    /// If a path is expected to represent a [`Container`](crate::project::resources::Container)
    /// but does not.
    #[error("`{0}` is not a Container")]
    PathNotAContainer(PathBuf),

    /// If two Containers with the same parent have the same name.
    #[error("clashing Container names")]
    ContainerNameConflict,
}

// *******************
// *** Asset Error ***
// *******************

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize, Error, Debug)]
pub enum AssetError {
    #[error("`{0}` is not a Container")]
    PathNotAContainer(PathBuf),

    #[error("`{0}` is already an Asset")]
    FileAlreadyAsset(PathBuf),

    #[error("`{0}` not found")]
    ContainerNotFound(PathBuf),

    #[error("`{0}` is invalid: {1}")]
    InvalidPath(PathBuf, String),

    /// The [`AssetFileAction`](crate::types::AssetFileAction) is
    /// incompatible with the path.
    #[error("invalid action: {0}")]
    IncompatibleAction(String),

    /// An error occured in the process of using the
    /// [`AssetBuilder`](crate::project::asset::AssetBuilder).
    #[error("builder errored: {0}")]
    BuilderError(String),
}

// ********************
// *** Users Errors ***
// ********************

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum UsersError {
    #[error("email `{0}` already exists")]
    DuplicateEmail(String),

    #[error("`{0}` is not a valid email")]
    InvalidEmail(String),
}

// // ****************************
// // *** Resource Store Error ***
// // ****************************

// #[cfg(feature = "fs")]
// #[derive(Serialize, Deserialize, Error, Debug)]
// pub enum ResourceStoreError {
//     /// If a [`ResourceId`] is expected to be present as a map key, but is not.
//     #[error("resource with id `{0}` is not present")]
//     IdNotPresent(ResourceId),

//     /// If trying to get an empty value.
//     #[
//     LoadEmptyValue,

//     /// If trying to set a value but the resource is already loaded.
//     ResourceAlreadyLoaded,
// }

// *******************
// *** Local Error ***
// *******************

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum Error {
    #[error("{0}")]
    CoreError(CoreError),

    #[error("{0}")]
    InvalidPath(PathBuf),

    #[error("{0}")]
    SettingsValidationError(SettingsValidationError),

    #[error("{0}")]
    UsersError(UsersError),

    #[cfg(feature = "fs")]
    #[error("{0}")]
    LoadError(LoadError),

    #[cfg(feature = "fs")]
    #[error("{0:?}")]
    LoaderErrors(LoaderErrors),

    #[cfg(feature = "fs")]
    #[error("{0}")]
    AssetError(AssetError),

    #[cfg(feature = "fs")]
    #[error("{0}")]
    ContainerError(ContainerError),

    #[cfg(feature = "fs")]
    #[error("{0}")]
    ProjectError(ProjectError),

    #[cfg(feature = "fs")]
    #[error("{0}")]
    SettingsFileError(SettingsFileError),
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::CoreError(err)
    }
}

#[cfg(feature = "fs")]
impl From<LoadError> for Error {
    fn from(err: LoadError) -> Self {
        Error::LoadError(err)
    }
}

#[cfg(feature = "fs")]
impl From<LoaderErrors> for Error {
    fn from(err: LoaderErrors) -> Self {
        Error::LoaderErrors(err)
    }
}

#[cfg(feature = "fs")]
impl From<AssetError> for Error {
    fn from(err: AssetError) -> Self {
        Error::AssetError(err)
    }
}

#[cfg(feature = "fs")]
impl From<ContainerError> for Error {
    fn from(err: ContainerError) -> Self {
        Error::ContainerError(err)
    }
}

#[cfg(feature = "fs")]
impl From<ProjectError> for Error {
    fn from(err: ProjectError) -> Self {
        Error::ProjectError(err)
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

// *******************
// *** Thot Result ***
// *******************

pub type Result<T = ()> = StdResult<T, Error>;

impl From<Error> for Result {
    fn from(err: Error) -> Self {
        Err(err)
    }
}

/// Copy of [`io::ErrorKind`] for `serde` de/serialization.
#[non_exhaustive]
#[derive(Serialize, Deserialize)]
#[serde(remote = "io::ErrorKind")]
pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    FilesystemQuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
}

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
