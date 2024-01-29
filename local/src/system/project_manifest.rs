//! High level functions associated to the projects list.
use super::collections::project_manifest::ProjectManifest;
use crate::error::IoSerde as IoSerdeError;
use std::fs;
use std::path::Path;

/// Adds a [`Project`] to the registry collection.
///
/// # See also
/// + `insert_project`
pub fn register_project(path: impl AsRef<Path>) -> Result<(), IoSerdeError> {
    let path = fs::canonicalize(path.as_ref())?;
    let mut projects = ProjectManifest::load_or_default()?;

    if !projects.contains(&path) {
        projects.push(path);
        projects.save()?;
    }

    Ok(())
}

/// Deregister a [`Project`].
pub fn deregister_project(path: impl AsRef<Path>) -> Result<(), IoSerdeError> {
    let path = fs::canonicalize(path.as_ref())?;
    let mut projects = ProjectManifest::load()?;
    projects.remove(&path);
    projects.save()?;
    Ok(())
}
