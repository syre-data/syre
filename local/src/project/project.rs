//! Functionality and resources related to projects.
use super::resources::{Project, Scripts};
use crate::common;
use crate::error::{Error, ProjectError, Result};
use crate::system::collections::Projects;
use crate::system::projects;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thot_core::error::{Error as CoreError, ProjectError as CoreProjectError, ResourceError};
use thot_core::project::Project as CoreProject;
use thot_core::types::ResourceId;

// ************
// *** Init ***
// ************

/// Initialize a new Thot project.
/// If the path is already initialized as a Thot resource -- i.e. has a `.thot` folder -- nothing is
/// done.
///
/// # Steps
/// 1. Create `.thot` folder to store data.
/// 2. Create [`Project`] for project info.
/// 3. Create `ProjectSettings` for project settings.
/// 4. Create `Script`s registry.
/// 5. Add [`Project`] to collections registry.
pub fn init(path: impl AsRef<Path>) -> Result<ResourceId> {
    let path = path.as_ref();
    if path_is_resource(path) {
        // project already initialized
        let rid = match project_id(path)? {
            Some(rid) => rid,
            None => {
                return Err(ProjectError::PathNotRegistered(path.to_path_buf()).into());
            }
        };

        return Ok(rid);
    }

    // create directory
    let thot_dir = common::thot_dir_of(path);
    fs::create_dir(&thot_dir)?;

    // create thot files
    let project = Project::new(path.into())?;
    project.save()?;

    let scripts = Scripts::new(path.into());
    scripts.save()?;

    projects::register_project(project.rid.clone(), project.base_path().into())?;
    Ok(project.rid.clone().into())
}

/// Creates a new Thot project.
/// Errors if the folder already exists.
///
/// # See also
/// + `init`
pub fn new(root: &Path) -> Result<ResourceId> {
    if root.exists() {
        return Err(io::Error::new(io::ErrorKind::IsADirectory, "folder already exists").into());
    }

    fs::create_dir_all(root)?;
    init(root)
}

/// Move project to a new location.
pub fn mv(rid: &ResourceId, to: &Path) -> Result {
    let mut projects = Projects::load()?;
    let Some(project) = projects.get_mut(rid) else {
        return Err(CoreError::ResourceError(ResourceError::does_not_exist(
            "`Project` is not registered",
        ))
        .into());
    };

    // move folder
    if let Err(err) = fs::rename(project, to) {
        return Err(err.into());
    }

    projects.save()?;
    Ok(())
}

/// Returns whether the given path is part of a Thot project.
///
/// # Returns
/// `true`` if the path has a <THOT_DIR> folder in it.
///
/// # Note
/// + Only works with `Container`s and `Project`s, not `Asset`s.
pub fn path_is_resource(path: &Path) -> bool {
    let path = common::thot_dir_of(path);
    path.exists()
}

/// Returns whether the given path is a project root,
/// i.e. has a <THOT_DIR>/<PROJECT_FILE>.
pub fn path_is_project_root(path: &Path) -> bool {
    let path = common::project_file_of(path);
    path.exists()
}

/// Returns path to the project root.
///
/// # See also
/// + [`project_resource_root_path`]
pub fn project_root_path(path: &Path) -> Result<PathBuf> {
    let o_path = PathBuf::from(path);
    let mut path = path.join("tmp"); // false join to pop off in loop
    while path.pop() {
        if !path_is_project_root(&path) {
            continue;
        }

        // TODO[h] Should not create.
        let prj = Project::load_from(path.clone())?;
        if prj.meta_level == 0 {
            return Ok(fs::canonicalize(path)?);
        }
    }

    Err(Error::ProjectError(ProjectError::PathNotInProject(o_path)))
}

/// Returns path to the project root for a Thot resource.
/// The entire path from start to the root of the project must follow resources.
/// i.e. If the path from start to root contains a folder that is not initiailized
/// as a Container, an error will be returned.
///
/// # See also
/// + [`project_root_path`]
pub fn project_resource_root_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    if !path_is_resource(path) {
        return Err(Error::ProjectError(ProjectError::PathNotInProject(
            PathBuf::from(path),
        )));
    }

    let mut path = path.join("tmp"); // false join to pop off in loop
    while path.pop() {
        let prj_file = common::project_file_of(&path);
        if !prj_file.exists() {
            // folder is not root
            continue;
        }

        let Ok(prj_json) = fs::read_to_string(prj_file) else {
            // TODO Handle metalevel.
            // Currently assumed that if project file can't be read, it is because
            // the file is being controlled by another process, likely the database
            // so just return the path.
            return Ok(fs::canonicalize(path)?);
        };

        let prj: CoreProject = match serde_json::from_str(prj_json.as_str()) {
            Ok(prj) => prj,
            Err(err) => return Err(err.into()),
        };

        if prj.meta_level == 0 {
            return Ok(fs::canonicalize(path)?);
        }
    }

    Err(CoreError::ProjectError(CoreProjectError::misconfigured("project has no root.")).into())
}

/// # Returns
/// + [`ResourceId`] of the containing [`Project`] if it exists.
/// + `None` if the path is not inside a `Project``.
pub fn project_id(path: impl AsRef<Path>) -> Result<Option<ResourceId>> {
    let root = match project_resource_root_path(path.as_ref()) {
        Ok(root) => root,
        Err(Error::ProjectError(ProjectError::PathNotInProject(_))) => return Ok(None),
        Err(err) => return Err(err),
    };

    projects::get_id(root.as_path())
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
