//! Functionality and resources related to projects.
use super::resources::{Project, ProjectSettings};
use crate::common;
use crate::constants::THOT_DIR;
use crate::error::ProjectError;
use crate::system::collections::Projects;
use crate::system::projects;
use crate::system::resources::project::Project as SystemProject;
use crate::{Error, Result};
use settings_manager::local_settings::{LocalSettings, LockSettingsFile};
use settings_manager::system_settings::SystemSettings;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ProjectError as CoreProjectError, ResourceError};
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
pub fn init(root: &Path) -> Result<ResourceId> {
    if path_is_resource(root) {
        // project already initialized
        let prj = project_registration(root)?;
        return Ok(prj.rid.into());
    }

    // create directory
    let thot_dir = root.join(THOT_DIR);
    fs::create_dir(&thot_dir)?;

    // create thot files
    // project
    let name = match root.file_name() {
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

    let mut project = Project::new(name.as_str())?;
    project.set_base_path(root.to_path_buf())?;
    project.acquire_lock()?;
    project.save()?;

    // project settings
    let mut settings = ProjectSettings::new();
    settings.set_base_path(root.to_path_buf())?;
    settings.acquire_lock()?;
    settings.save()?;

    // add project to collection registry
    let prj = SystemProject::new(project.rid.clone().into(), PathBuf::from(root));
    projects::register_project(prj)?;

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
    let mut projects = Projects::load()?;
    let Some(project) = projects.get_mut(rid) else {
        return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` is not registered")).into());
    };

    // move folder
    if let Err(err) = fs::rename(&project.path, to) {
        return Err(err.into());
    }

    project.path = to.to_path_buf();
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
/// + `project_root_path`
pub fn project_root_path(path: &Path) -> Result<PathBuf> {
    let o_path = PathBuf::from(path);
    let mut path = path.join("tmp"); // false join to pop off in loop
    while path.pop() {
        if !path_is_project_root(&path) {
            continue;
        }

        let prj = Project::load(&path)?;
        if prj.meta_level == 0 {
            return common::canonicalize_path(path);
        }
    }

    Err(Error::ProjectError(ProjectError::PathNotInProject(o_path)))
}

/// Returns path to the project root for a thot resource.
/// The entire path from start to the root of the project must follow resources.
/// i.e. If the path from start to root contains a folder that is not initiailized
/// as a Container, an error will be returned.
///
/// # See also
/// + `project_root_path`
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

        let prj_json = match fs::read_to_string(prj_file) {
            Ok(json) => json,
            Err(err) => return Err(err.into()),
        };

        let prj: Project = match serde_json::from_str(prj_json.as_str()) {
            Ok(prj) => prj,
            Err(err) => return Err(err.into()),
        };

        if prj.meta_level == 0 {
            return common::canonicalize_path(path);
        }
    }

    Err(CoreError::ProjectError(CoreProjectError::Misconfigured("project has no root.")).into())
}

/// Returns registration info on the project root of the given path.
pub fn project_registration(path: &Path) -> Result<SystemProject> {
    let root = project_resource_root_path(path)?;
    let reg = projects::project_by_path(root.as_path())?;
    let Some(reg) = reg else {
        let rid = None; // @todo: Manually get rid from root path if it exists.
        return Err(
            CoreError::ProjectError(CoreProjectError::NotRegistered(rid, Some(root))).into(),
        );
    };

    Ok(reg.clone())
}

// @todo: Check to see if can be removed.
//  Should likely be replaced with `Project::load`.
//
// /// Loads a project from a path.
// /// The path should be the folder containing the THOT_DIR folder.
// pub fn load_project(path: &Path) -> Result<Project> {
//     let thot_dir = PathBuf::from(path).join(THOT_DIR);
//     if !thot_dir.exists() {
//         // exited project
//         return Err(Error::ProjectError(ProjectError::PathNotInProject(
//             PathBuf::from(path),
//         )));
//     }

//     let prj_file = thot_dir.join(PROJECT_FILE);
//     if !prj_file.exists() {
//         // folder is not root
//         return Err(Error::ProjectError(ProjectError::PathNotAProjectRoot(
//             PathBuf::from(path),
//         )));
//     }

//     let prj_json = match fs::read_to_string(prj_file) {
//         Ok(json) => json,
//         Err(err) => return Err(err.into()),
//     };

//     let prj: Project = match serde_json::from_str(prj_json.as_str()) {
//         Ok(prj) => prj,
//         Err(err) => return Err(err.into()),
//     };

//     Ok(prj)
// }

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
