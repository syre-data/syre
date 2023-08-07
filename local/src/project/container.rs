//! High level functionality related to Containers.
use super::project;
use super::resources::Container;
use crate::common::{container_file_of, thot_dir_of};
use crate::error::ContainerError;
use crate::{Error, Result};
use std::path::{Path, PathBuf};
use std::{fs, io};
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;

/// Create a new Container, returning the [`ResourceId`].
/// Creates directories as needed.
///
/// # Errors
/// + [`io::ErrorKind::IsADirectory`]: A directory at the given path already exists.
///
/// # See also
/// + [`init`]
pub fn new(path: &Path) -> Result<ResourceId> {
    if path.exists() {
        return Err(io::Error::new(io::ErrorKind::IsADirectory, "path already exists").into());
    }

    fs::create_dir_all(path)?;
    init(path)
}

/// Initialize folder as a [`Container`](CoreContainer).
///
/// # Errors
/// + [`io::ErrorKind::NotADirectory`]: The given path does not exist.
///
/// # See also
/// + [`init_from`]
pub fn init(path: &Path) -> Result<ResourceId> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotADirectory, "path does not exist").into());
    }

    // check if path is already a resource
    if project::path_is_resource(path) {
        if path_is_container(path) {
            // path is already a container
            // return resource id
            let container = Container::load_from(path)?;
            return Ok(container.rid.clone());
        }
    } else {
        // path is not a thot resource,
        // initialize as resource
        let thot_dir = thot_dir_of(path);
        fs::create_dir(&thot_dir)?;
    }

    // initialize container
    // assets included
    let mut container = Container::new(path);
    container.save()?;
    Ok(container.rid.clone())
}

/// Initializes folder as a [`Container`](CoreContainer)
/// with initial values.
///
/// # See also
/// + [`init`]
pub fn init_from(path: &Path, container: CoreContainer) -> Result {
    init(path)?;
    let mut cont = Container::load_from(path)?;
    *cont = container;
    cont.save()?;

    Ok(())
}

/// Move a Container, including all its resources (children, assets, settings,
/// etc.) a to a new location.
pub fn mv(rid: ResourceId, to: &Path) -> Result {
    todo!();
}

/// Remove a folder as a Container without deleting it.
///
/// # See also
/// + `delete'
pub fn remove(rid: ResourceId) -> Result {
    todo!();
}

/// Delete a Container and all its resources (children, assets, settings, etc.).
///
/// # Notes
/// This will permananently delete all the associated resources from the file system.
/// For a non-destructive way to remove a Container and its resource see `remove`.
///
/// # See also
/// + `remove`
pub fn delete(rid: ResourceId) -> Result {
    todo!();
}

/// Updates the Container with the matching resource id.
pub fn update(container: Container) -> Result {
    todo!();
}

/// Returns whether or not the path is a Container.
/// Checks if <path>/<THOT_DIR>/<CONTAINER_FILE> exists.
pub fn path_is_container(path: &Path) -> bool {
    let c_path = container_file_of(path);
    c_path.exists()
}

/// Initialize an existing folder as a child.
/// Initializes the child folder as a Container, then registers it with its parent as a child.
///
/// # Arguments
/// + `child`: Path to the child directory.
/// + `container`: Path to the parent Container, or None to use the child's parent directory.
///
/// # Errors
/// + If `child` is not a child of the `container` directory.
///
/// # Notes
/// + Currently `child` must be a child of the `container` directory.
///     Both are provided for future proofing if children are ever allowed to
///     not be children of their parent Container's directory.
///
/// # See also
/// + `new_child`
pub fn init_child(child: &Path, container: Option<&Path>) -> Result<ResourceId> {
    // check child is valid
    if !child.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            "child path is not a directory",
        )
        .into());
    }

    let parent = match child.parent() {
        Some(p) => p,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "invalid path for container",
            )
            .into())
        }
    };

    let container = match container {
        None => parent,
        Some(p) => p,
    };

    // ensure container and parent are the same
    // can be removed if restriction is lifted
    if parent != container {
        return Err(Error::ContainerError(ContainerError::InvalidChildPath(
            PathBuf::from(child),
        )));
    }

    if !path_is_container(container) {
        return Err(Error::ContainerError(ContainerError::PathNotAContainer(
            PathBuf::from(container),
        )));
    }

    // init and register
    let rid = init(child)?;
    let mut container = Container::load_from(container)?;
    container.save()?;

    Ok(rid)
}

/// Create a new folder at the given path, initialize it as a [`Container`],
/// and add it as a child to the given parent.
///
/// # Arguments
/// + `child`: Path to the child directory.
/// + `container`: Path to the parent `Container`, or `None` to use the child's parent directory.
///
/// # See also
/// + `init_child`
pub fn new_child(child: &Path, container: Option<&Path>) -> Result<ResourceId> {
    fs::create_dir(child)?;
    init_child(child, container)
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
