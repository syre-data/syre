//! Functionality and resources related to projects.
use super::resources::project::{Loader, Project};
use crate::common;
use crate::constants::THOT_DIR;
use crate::error::ProjectError;
use crate::system::collections::Projects;
use crate::system::projects;
use crate::{Error, Result};
use settings_manager::{system_settings::Loader as SystemLoader, Settings};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ProjectError as CoreProjectError, ResourceError};
use thot_core::project::Project as CoreProject;
use thot_core::types::ResourceId;

// ************
// *** Init ***
// ************

// @todo: reinitialize project if already a project. See Git's functionality.
/// Initialize a new Thot project.
/// If the path is already initialized as a Thot resource -- i.e. has a `.thot` folder -- nothing is
/// done.
///
/// # Steps
/// 1. Create `.thot` folder to store data.
/// 2. Create [`Project`] for project info.
/// 3. Create [`ProjectSettings`] for project settings.
/// 4. Create `Script`s registry.
/// 5. Add [`Project`] to collections registry.
pub fn init(path: &Path) -> Result<ResourceId> {
    if path_is_resource(path) {
        // project already initialized
        let rid = project_id(path)?.expect("path is unregistered `Project`");
        return Ok(rid);
    }

    // create directory
    let thot_dir = path.join(THOT_DIR);
    fs::create_dir(&thot_dir)?;

    // create thot files
    // project
    let name = match path.file_name() {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, // @todo: Should be InvalidFilename
                "file name could not be extracted from path",
            )
            .into());
        }
        Some(f_name) => {
            match f_name.to_str() {
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput, // @todo: Should be InvalidFilename
                        "file name could not be converted to string",
                    )
                    .into());
                }
                Some(f_str) => String::from(f_str),
            }
        }
    };

    let mut project: Project = Loader::load_or_create(path.into())?.into();
    project.name = name;
    project.save()?;

    // add project to collection registry
    projects::register_project(project.rid, project.base_path().into())?;

    // success
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
    let mut projects: Projects = SystemLoader::load_or_create::<Projects>()?.into();
    let Some(project) = projects.get_mut(rid) else {
        return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` is not registered")).into());
    };

    // move folder
    if let Err(err) = fs::rename(project, to) {
        return Err(err.into());
    }

    projects.save()?;
    Ok(())
}

/// Returns whether the given path is part of a Thot project.
/// Returns true if the path has a <THOT_DIR> folder in it.
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

        let prj: Project = Loader::load_or_create(path.into())?.into();
        if prj.meta_level == 0 {
            return common::canonicalize_path(path);
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
pub fn project_resource_root_path(path: &Path) -> Result<PathBuf> {
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
            // @todo: Handle metalevel.
            // Currently assumed that if project file can't be read, it is because
            // the file is being controlled by another process, likely the database
            // so just return the path.
            return common::canonicalize_path(path);
        };

        let prj: CoreProject = match serde_json::from_str(prj_json.as_str()) {
            Ok(prj) => prj,
            Err(err) => return Err(err.into()),
        };

        if prj.meta_level == 0 {
            return common::canonicalize_path(path);
        }
    }

    Err(CoreError::ProjectError(CoreProjectError::Misconfigured("project has no root.")).into())
}

/// Returns the [`ResourceId`] of the containing [`Project`] if it exists..
pub fn project_id(path: &Path) -> Result<Option<ResourceId>> {
    let root = project_resource_root_path(path)?;
    projects::project_by_path(root.as_path())
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
