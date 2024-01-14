//! Commands related to projects.
use crate::error::{DesktopSettings as DesktopSettingsError, Result};
use crate::state::AppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use tauri::State;
use thot_core::error::{Error as CoreError, Project as ProjectError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::project::{Container, Project};
use thot_core::types::ResourceId;
use thot_desktop_lib::error::Analysis as AnalysisError;
use thot_local::project::project;
use thot_local::project::resources::Project as LocalProject;
use thot_local::system::projects as sys_projects;
use thot_local::types::ProjectSettings;
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
    let projects = db
        .send(ProjectCommand::LoadUser(user).into())
        .expect("could not load user `Project`s");

    let projects: DbResult<Vec<(Project, ProjectSettings)>> = serde_json::from_value(projects)
        .expect("could not convert `GetUserProjects` result to `Vec<(Project, ProjectSettings)>`");

    Ok(projects?)
}

// ********************
// *** load project ***
// ********************

/// Loads a [`Project`].
#[tauri::command]
pub fn load_project(db: State<DbClient>, path: PathBuf) -> DbResult<(Project, ProjectSettings)> {
    let project = db
        .send(ProjectCommand::LoadWithSettings(path).into())
        .expect("could not load `Project`");

    serde_json::from_value::<DbResult<(Project, ProjectSettings)>>(project).unwrap()
}

// *******************
// *** add project ***
// *******************

/// Adds an existing [`Project`] to the users vault.
#[tauri::command]
pub fn add_project(
    app_state: State<AppState>,
    db: State<DbClient>,
    path: PathBuf,
) -> Result<(Project, ProjectSettings)> {
    let user = app_state
        .user
        .lock()
        .expect("could not lock app state `User`");

    let Some(user) = user.as_ref() else {
        return Err(DesktopSettingsError::NoUser.into());
    };

    let project = db
        .send(ProjectCommand::Add(path, user.rid.clone()).into())
        .expect("could not add `Project`");

    let project = serde_json::from_value::<DbResult<(Project, ProjectSettings)>>(project).unwrap();
    Ok(project?)
}

// *******************
// *** get project ***
// *******************

/// Gets a [`Project`].
#[tauri::command]
pub fn get_project(db: State<DbClient>, rid: ResourceId) -> Result<Option<Project>> {
    let project = db
        .send(ProjectCommand::Get(rid).into())
        .expect("could not get `Project`");

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

// ********************
// *** init project ***
// ********************

/// Initializes a new project.
#[tauri::command]
pub fn init_project(path: &Path) -> Result<ResourceId> {
    let rid = project::init(path)?;

    // create analysis folder
    let analysis_root = "analysis";
    let mut analysis = path.to_path_buf();
    analysis.push(analysis_root);
    fs::create_dir(&analysis).expect("could not create analysis directory");

    let mut project = LocalProject::load_from(path)?;
    project.analysis_root = Some(PathBuf::from(analysis_root));
    project.save()?;

    Ok(rid)
}

#[tauri::command]
pub fn init_project_from(path: &Path) -> thot_local::Result<ResourceId> {
    thot_local::project::init(path, "data", "analysis")
}

// ************************
// *** get project path ***
// ************************

#[tauri::command]
pub fn get_project_path(rid: ResourceId) -> Result<PathBuf> {
    let Some(path) = sys_projects::get_path(&rid)? else {
        return Err(CoreError::Project(ProjectError::NotRegistered(
            Some(ResourceId::from(rid.clone())),
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
#[tracing::instrument(skip(db))]
#[tauri::command]
pub fn update_project(db: State<DbClient>, project: Project) -> DbResult {
    let res = db
        .send(ProjectCommand::Update(project).into())
        .expect("could not update `Project`");

    serde_json::from_value(res).unwrap()
}

// ***************
// *** analyze ***
// ***************

#[tauri::command]
pub fn analyze(
    db: State<DbClient>,
    root: ResourceId,
    max_tasks: Option<usize>,
) -> StdResult<(), AnalysisError> {
    let graph = match db.send(GraphCommand::Get(root.clone()).into()) {
        Ok(graph) => graph,
        Err(err) => {
            return Err(AnalysisError::ZMQ(format!("{err:?}")));
        }
    };

    let graph: Option<ResourceTree<Container>> = serde_json::from_value(graph).unwrap();
    let Some(mut graph) = graph else {
        return Err(AnalysisError::GraphNotFound);
    };

    let runner = Runner::new();
    match max_tasks {
        None => runner.from(&mut graph, &root)?,
        Some(max_tasks) => runner.with_tasks(&mut graph, max_tasks)?,
    }

    Ok(())
}
