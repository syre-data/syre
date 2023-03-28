//! High level functions associated to the projects list.
use super::collections::projects::Projects;
use super::resources::project::Project;
use crate::error::{Error, ProjectError, Result, SettingsValidationError};
use crate::system::settings::UserSettings;
use settings_manager::SystemSettings;
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::types::ResourceId;

// ****************
// *** Projects ***
// ****************

/// Adds a [`Project`] to the registry collection.
pub fn register_project(project: Project) -> Result {
    let mut projects = Projects::load_or_default()?;
    let rid = project.rid.clone();

    // check if project is already registered.
    if projects.contains_key(&rid) {
        return Err(Error::CoreError(CoreError::ResourceError(
            ResourceError::DuplicateId(project.rid.into()),
        )));
    }

    projects.insert(rid, project);
    projects.save()?;

    Ok(())
}

/// Deregister a [`Project`].
pub fn deregister_project(id: &ResourceId) -> Result {
    let mut projects = Projects::load_or_default()?;
    projects.remove(&id);
    projects.save()?;
    Ok(())
}

/// Retrieves a [`Project`] by its [`ResourceId`].
/// Returns `None` if project is not found.
pub fn project_by_id(id: &ResourceId) -> Result<Option<Project>> {
    let projects = Projects::load_or_default()?;
    let project = projects.get(id);
    Ok(project.map(|p| p.clone()))
}

/// Returns a [`Project`] by its path.
/// Returns None if project is not found.
pub fn project_by_path(path: &Path) -> Result<Option<Project>> {
    let projects = Projects::load_or_default()?;
    let projects = &projects
        .values()
        .filter(|prj| prj.path == path)
        .collect::<Vec<&Project>>();

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
pub fn update_project(project: Project) -> Result {
    let mut projects = Projects::load_or_default()?;
    projects.insert(project.rid.clone(), project);

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

    let mut settings = UserSettings::load_or_default()?;
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

    let mut settings = UserSettings::load_or_default()?;
    settings.active_project = Some(project.rid);
    settings.save()?;
    Ok(())
}

pub fn unset_active_project() -> Result {
    let mut settings = UserSettings::load_or_default()?;
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
