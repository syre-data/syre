//! Commands related to projects.
use crate::error::{DesktopSettingsError, Error, Result};
use crate::state::AppState;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use thot_core::error::{Error as CoreError, ProjectError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::project::{Container, Project};
use thot_core::types::ResourceId;
use thot_desktop_lib::error::{Error as LibError, Result as LibResult};
use thot_local::project::project;
use thot_local::project::resources::project::{Loader as ProjectLoader, Project as LocalProject};
use thot_local::project::types::ProjectSettings;
use thot_local::system::projects as sys_projects;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::{GraphCommand, ProjectCommand};
use thot_local_database::Result as DbResult;
use thot_local_runner::Runner;

// **************************
// *** load user projects ***
// **************************

/// Loads all the active user's projects.
#[tauri::command]
pub fn load_user_projects(
    db: State<DbClient>,
    user: ResourceId,
) -> Result<Vec<(Project, ProjectSettings)>> {
    let projects = db.send(ProjectCommand::LoadUser(user).into());
    let projects: DbResult<Vec<(Project, ProjectSettings)>> = serde_json::from_value(projects)
        .expect("could not convert `GetUserProjects` result to `Vec<(Project, ProjectSettings)>`");

    Ok(projects?)
}

// ********************
// *** load project ***
// ********************

/// Loads a [`Project`].
#[tauri::command]
pub fn load_project(db: State<DbClient>, path: PathBuf) -> Result<(Project, ProjectSettings)> {
    let project = db.send(ProjectCommand::Load(path).into());
    let project: DbResult<(Project, ProjectSettings)> =
        serde_json::from_value(project).expect("could not convert `Load` result to `Project`");

    Ok(project?)
}

// *******************
// *** add project ***
// *******************

/// Adds a [`Project`].
#[tauri::command]
pub fn add_project(
    app_state: State<AppState>,
    db: State<DbClient>,
    path: PathBuf,
) -> LibResult<Project> {
    let user = app_state
        .user
        .lock()
        .expect("could not lock app state `User`");

    let Some(user) = user.as_ref() else {
        return Err(LibError::DatabaseError(format!("{:?}", Error::DesktopSettingsError(DesktopSettingsError::NoUser))));
    };

    let project = db.send(ProjectCommand::Add(path, user.rid.clone()).into());
    let project: DbResult<Project> =
        serde_json::from_value(project).expect("could not convert `Add` result to `Project`");

    project.map_err(|err| LibError::DatabaseError(format!("{:?}", err)))
}

// *******************
// *** get project ***
// *******************
/// Gets a [`Project`].
#[tauri::command]
pub fn get_project(db: State<DbClient>, rid: ResourceId) -> Result<Option<Project>> {
    let project = db.send(ProjectCommand::Get(rid).into());
    let project: Option<Project> = serde_json::from_value(project)
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
// pub fn new_project(app_state: State<AppState>, name: &str) -> Result<Project> {
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

    // @todo: Move to frontend.
    // create analysis folder
    let analysis_root = "analysis";
    let mut analysis = path.to_path_buf();
    analysis.push(analysis_root);
    fs::create_dir(&analysis).expect("could not create analysis directory");

    let mut project: LocalProject = ProjectLoader::load_or_create(path)
        .expect("Could not load `Project`")
        .into();

    project.analysis_root = Some(PathBuf::from(analysis_root));
    project.save()?;

    Ok(rid)
}

// ************************
// *** get project path ***
// ************************

#[tauri::command]
pub fn get_project_path(id: ResourceId) -> Result<PathBuf> {
    let Some(path) = sys_projects::get_path(&id)? else {
        return Err(CoreError::ProjectError(ProjectError::NotRegistered(
            Some(ResourceId::from(id.clone())),
            None,
        ))
        .into());
    };

    Ok(path)
}

// **********************
// *** update project ***
// **********************

/// Updates a project.
#[tauri::command]
pub fn update_project(db: State<DbClient>, project: Project) -> Result {
    let res = db.send(ProjectCommand::Update(project).into());
    let res: DbResult = serde_json::from_value(res).expect("could not convert from `Update`");

    Ok(res?)
}

// ***************
// *** analyze ***
// ***************

#[tauri::command]
pub fn analyze(db: State<DbClient>, root: ResourceId, max_tasks: Option<usize>) -> LibResult {
    let graph = db.send(GraphCommand::Get(root).into());
    let graph: Option<ResourceTree<Container>> =
        serde_json::from_value(graph).expect("could not convert from `Get` to `Container` tree");

    let Some(mut graph) = graph else {
        let error = CoreError::ResourceError(ResourceError::DoesNotExist("root `Container` not loaded"));
        return Err(LibError::DatabaseError(format!("{error:?}")));
    };

    let runner = Runner::new();
    let res = match max_tasks {
        None => runner.run(&mut graph),
        Some(max_tasks) => runner.run_with_tasks(&mut graph, max_tasks),
    };

    if res.is_err() {
        return Err(LibError::DatabaseError(format!("{res:?}")));
    }

    Ok(())
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
