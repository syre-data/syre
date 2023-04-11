//! High level functions associated to the projects list.
use super::collections::projects::Projects;
use crate::error::{Error, ProjectError, Result, SettingsValidationError};
use crate::system::settings::user_settings::UserSettings;
use settings_manager::system_settings::Loader;
use settings_manager::Settings;
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::types::ResourceId;

// ****************
// *** Projects ***
// ****************

/// Adds a [`Project`] to the registry collection.
///
/// # Errors
/// + [`ResourceError::DuplicateId`] if the `Project` is already registered.
///
/// # See also
/// + `insert_project`
pub fn register_project(rid: ResourceId, path: PathBuf) -> Result {
    let mut projects: Projects = Loader::load_or_create::<Projects>()?.into();

    // check if project is already registered.
    if projects.contains_key(&rid) {
        return Err(Error::CoreError(CoreError::ResourceError(
            ResourceError::DuplicateId(rid.into()),
        )));
    }

    projects.insert(rid, path);
    projects.save()?;

    Ok(())
}

/// Deregister a [`Project`].
pub fn deregister_project(id: &ResourceId) -> Result {
    let mut projects: Projects = Loader::load_or_create::<Projects>()?.into();
    projects.remove(&id);
    projects.save()?;
    Ok(())
}

/// Retrieves a [`Project`] by its [`ResourceId`].
/// Returns `None` if project is not found.
pub fn project_by_id(id: &ResourceId) -> Result<Option<PathBuf>> {
    let projects: Projects = Loader::load_or_create::<Projects>()?.into();
    Ok(projects.get(id).cloned())
}

/// Returns a [`Project`] by its path.
/// Returns None if project is not found.
pub fn project_by_path(path: &Path) -> Result<Option<ResourceId>> {
    let projects: Projects = Loader::load_or_create::<Projects>()?.into();
    let projects = &projects
        .iter()
        .filter_map(
            |(rid, p_path)| {
                if p_path == path {
                    Some(rid)
                } else {
                    None
                }
            },
        )
        .collect::<Vec<&ResourceId>>();

    match projects.len() {
        0 => Ok(None),
        1 => Ok(Some(projects[0].clone())),
        _ => Err(Error::ProjectError(ProjectError::DuplicatePath(
            PathBuf::from(path),
        ))),
    }
}

/// Updates a [`Project`].
/// Replaces the project in the projects collection with the same id.
pub fn insert_project(rid: ResourceId, path: PathBuf) -> Result {
    let mut projects: Projects = Loader::load_or_create::<Projects>()?.into();
    projects.insert(rid, path);

    projects.save()?;
    Ok(())
}

pub fn set_active_project(id: &ResourceId) -> Result {
    // ensure valid project
    if !validate_project(id) {
        return Err(Error::SettingsValidationError(
            SettingsValidationError::InvalidSetting,
        ));
    };

    let mut settings: UserSettings = Loader::load_or_create::<UserSettings>()?.into();
    settings.active_project = Some((*id).clone().into());
    settings.save()?;
    Ok(())
}

pub fn set_active_project_by_path(path: &Path) -> Result {
    let project = match project_by_path(path)? {
        None => {
            return Err(Error::ProjectError(ProjectError::PathNotAProjectRoot(
                PathBuf::from(path),
            )))
        }
        Some(p) => p,
    };

    let mut settings: UserSettings = Loader::load_or_create::<UserSettings>()?.into();
    settings.active_project = Some(project);
    settings.save()?;
    Ok(())
}

pub fn unset_active_project() -> Result {
    let mut settings: UserSettings = Loader::load_or_create::<UserSettings>()?.into();
    settings.active_project = None;
    settings.save()?;
    Ok(())
}

// *************************
// *** private functions ***
// *************************

// @todo
fn validate_project(id: &ResourceId) -> bool {
    true
}

// @todo
fn validate_project_path(path: &Path) -> bool {
    true
}

#[cfg(test)]
#[path = "./projects_test.rs"]
mod projects_test;
