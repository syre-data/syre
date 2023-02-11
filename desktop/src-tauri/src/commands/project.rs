//! Commands related to projects.
use crate::error::Result;
use crate::state::AppState;
use std::path::{Path, PathBuf};
use tauri::State;
use thot_core::error::{Error as CoreError, ProjectError};
use thot_core::project::Project as CoreProject;
use thot_core::types::ResourceId;
use thot_local::project::project;
use thot_local::system::projects as sys_projects;
use thot_local::system::resources::Project as SystemProject;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::ProjectCommand;
use thot_local_database::Result as DbResult;

/// Loads all the active user's projects.
///
/// # Returns
/// A tuple of loaded projects and projects that errored while loading,
/// with the associated error.
#[tauri::command]
pub fn load_user_projects(db: State<DbClient>, user: ResourceId) -> Result<Vec<CoreProject>> {
    let projects = db.send(ProjectCommand::LoadUser(user).into());
    let projects: DbResult<Vec<CoreProject>> = serde_json::from_value(projects)
        .expect("could not convert `GetUserProjects` result to `Vec<Project>`");

    Ok(projects?)
}

// ********************
// *** load project ***
// ********************

/// Loads a [`Project`].
#[tauri::command]
pub fn load_project(db: State<DbClient>, path: PathBuf) -> Result<CoreProject> {
    let project = db.send(ProjectCommand::Load(path).into());
    let project: DbResult<CoreProject> = serde_json::from_value(project)
        .expect("could not convert `LoadProject` result to `Project`");

    Ok(project?)
}

// *******************
// *** get project ***
// *******************
/// Gets a [`Project`].
#[tauri::command]
pub fn get_project(db: State<DbClient>, rid: ResourceId) -> Result<Option<CoreProject>> {
    let project = db.send(ProjectCommand::Get(rid).into());
    let project: Option<CoreProject> = serde_json::from_value(project)
        .expect("could not convert `GetProject` result to `Project`");

    Ok(project)
}

// **************************
// *** set active project ***
// **************************

/// Set the active project.
/// Sets the active project on the [system settings](sys_projects::set_active_project).
/// Sets the active project `id` on the [`AppState`].
#[tauri::command]
pub fn set_active_project(app_state: State<AppState>, rid: Option<ResourceId>) -> Result {
    // system settings
    if let Some(rid) = rid.clone() {
        sys_projects::set_active_project(&rid)?;
    } else {
        sys_projects::unset_active_project()?;
    }

    // app state
    *app_state.active_project.lock().unwrap() = rid;

    Ok(())
}

// *******************
// *** new project ***
// *******************

// @todo: Can possibly remove.
//
// /// Creates a new project.
// #[tauri::command]
// pub fn new_project(app_state: State<AppState>, name: &str) -> Result<CoreProject> {
//     // create new project
//     let project = LocalProject::new(name)?;
//     let prj_props = (*project).clone();

//     // store project
//     let mut project_store = app_state
//         .projects
//         .lock()
//         .expect("could not lock `AppState.projects`");

//     project_store.insert(prj.properties.rid.clone(), prj);

//     Ok(prj_props)
// }

// ********************
// *** init project ***
// ********************

/// Initializes a new project.
#[tauri::command]
pub fn init_project(path: &Path) -> Result<ResourceId> {
    let rid = project::init(path)?;
    Ok(rid)
}

// ************************
// *** get project path ***
// ************************

#[tauri::command]
pub fn get_project_path(id: ResourceId) -> Result<PathBuf> {
    let prj_info = project_info(&id)?;
    Ok(prj_info.path)
}

// **********************
// *** update project ***
// **********************

/// Updates a project.
#[tauri::command]
pub fn update_project(db: State<DbClient>, project: CoreProject) -> Result {
    let res = db.send(ProjectCommand::Update(project).into());
    let res: DbResult =
        serde_json::from_value(res).expect("could not convert from `UpdateProject`");

    Ok(res?)
}

// ---------------
// --- helpers ---
// ---------------

// ********************
// *** project info ***
// ********************

fn project_info(id: &ResourceId) -> Result<SystemProject> {
    let prj_info = sys_projects::project_by_id(id)?;
    if prj_info.is_none() {
        return Err(CoreError::ProjectError(ProjectError::NotRegistered(
            Some(ResourceId::from(id.clone())),
            None,
        ))
        .into());
    }

    let prj_info = prj_info.expect("project should be some");
    Ok(prj_info)
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
