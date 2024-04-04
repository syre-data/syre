//! Commands related to projects.
use crate::error::{DesktopSettings as DesktopSettingsError, Error, Result};
use crate::state::AppState;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use std::{fs, io};
use syre_core::error::{Error as CoreError, Project as ProjectError};
use syre_core::graph::ResourceTree;
use syre_core::project::{Container, Project};
use syre_core::types::{ResourceId, UserPermissions};
use syre_desktop_lib::error::{Analysis as AnalysisError, RemoveResource as RemoveResourceError};
use syre_local::error::{
    Error as LocalError, IoSerde as IoSerdeError, Project as LocalProjectError,
};
use syre_local::project::project as local_project;
use syre_local::project::resources::Project as LocalProject;
use syre_local::system::collections::ProjectManifest;
use syre_local::system::project_manifest as sys_projects;
use syre_local::types::ProjectSettings;
use syre_local_database::client::Client as DbClient;
use syre_local_database::command::{GraphCommand, ProjectCommand};
use syre_local_database::error::server::{
    LoadUserProjects as LoadUserProjectsError, Update as UpdateError,
};
use syre_local_database::Result as DbResult;
use syre_local_runner::Runner;
use tauri::State;

/// Loads all the active user's projects.
#[tauri::command]
pub fn load_user_projects(
    db: State<DbClient>,
    user: ResourceId,
) -> StdResult<Vec<(Project, ProjectSettings)>, LoadUserProjectsError> {
    db.project().load_user(user).unwrap()
}

/// Loads a [`Project`].
#[tauri::command]
pub fn load_project(
    db: State<DbClient>,
    path: PathBuf,
) -> StdResult<(Project, ProjectSettings), IoSerdeError> {
    db.project().load_with_settings(path).unwrap()
}

/// Imports an existing [`Project`].
/// Adds the active user to it.
#[tauri::command]
pub fn import_project(
    db: State<DbClient>,
    app_state: State<AppState>,
    path: PathBuf,
) -> Result<(Project, ProjectSettings)> {
    let path = fs::canonicalize(path)?;
    let user = app_state.user.lock().unwrap();
    let Some(user) = user.as_ref() else {
        return Err(DesktopSettingsError::NoUser.into());
    };

    let mut project = match LocalProject::load_from(&path) {
        Ok(project) => project,
        Err(err) => match err {
            IoSerdeError::Io(io::ErrorKind::NotFound) => {
                return Err(
                    LocalError::Project(LocalProjectError::PathNotAProjectRoot(path)).into(),
                )
            }
            _ => return Err(err.into()),
        },
    };

    project.settings_mut().permissions.insert(
        user.rid.clone(),
        UserPermissions::with_permissions(true, true, true),
    );

    project.save()?;
    let mut project_manifest = ProjectManifest::load_or_default()?;
    project_manifest.push(path.clone());
    project_manifest.save()?;

    Ok(db
        .project()
        .load_with_settings(project.base_path().to_path_buf())
        .unwrap()?)
}

/// Gets a [`Project`].
#[tauri::command]
pub fn get_project(db: State<DbClient>, rid: ResourceId) -> Option<Project> {
    db.project().get(rid).unwrap()
}

#[tauri::command]
pub fn delete_project(db: State<DbClient>, rid: ResourceId) -> StdResult<(), RemoveResourceError> {
    let path = match db.project().path(rid) {
        Ok(Some(path)) => path,

        Ok(None) => {
            return Err(RemoveResourceError::Database(
                "Could not get Project's path".to_string(),
            ))
        }

        Err(err) => return Err(RemoveResourceError::ZMQ(format!("{err:?}"))),
    };

    match trash::delete(path) {
        Ok(_) => Ok(()),
        Err(err) => todo!("{err:?}"),
    }
}

/// Initializes a new project.
#[tauri::command]
pub fn init_project(path: &Path) -> Result<ResourceId> {
    let rid = local_project::init(path)?;

    // create analysis folder
    let analysis_root = "analysis";
    let mut analysis = path.to_path_buf();
    analysis.push(analysis_root);
    fs::create_dir(&analysis).unwrap();

    let mut project = LocalProject::load_from(path)?;
    project.analysis_root = Some(PathBuf::from(analysis_root));
    project.save()?;

    Ok(rid)
}

#[tauri::command]
pub fn init_project_from(path: &Path) -> syre_local::Result<ResourceId> {
    let converter = local_project::converter::Converter::new();
    converter.convert(path)
}

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

/// Updates a project.
#[tauri::command]
pub fn update_project(db: State<DbClient>, project: Project) -> StdResult<(), UpdateError> {
    db.project().update(project).unwrap()
}

#[tauri::command]
pub fn analyze(
    db: State<DbClient>,
    root: ResourceId,
    max_tasks: Option<usize>,
) -> StdResult<(), AnalysisError> {
    let graph = match db.graph().get(root.clone()) {
        Ok(graph) => graph,
        Err(err) => {
            return Err(AnalysisError::ZMQ(format!("{err:?}")));
        }
    };

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
