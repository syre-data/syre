//! Resources for [`project commands`](thot_desktop_tauri::commands::project).
use super::common::{PathBufArgs, ResourceIdArgs};
use crate::common::invoke_result;
use serde::Serialize;
use std::path::PathBuf;
use thot_core::project::Project;
use thot_core::types::ResourceId;
use thot_desktop_lib::error::Analysis as AnalysisError;
use thot_local::types::ProjectSettings;
use thot_local_database::error::server::LoadUserProjects as LoadUserProjectsError;
use thot_local_database::Result as DbResult;

pub async fn init_project(path: PathBuf) -> Result<ResourceId, String> {
    invoke_result("init_project", PathBufArgs { path }).await
}

pub async fn init_project_from(path: PathBuf) -> thot_local::Result<ResourceId> {
    invoke_result("init_project_from", PathBufArgs { path }).await
}

pub async fn load_project(
    path: PathBuf,
) -> thot_local_database::Result<(Project, ProjectSettings)> {
    invoke_result("load_project", PathBufArgs { path }).await
}

pub async fn load_user_projects(
    user: ResourceId,
) -> Result<Vec<(Project, ProjectSettings)>, LoadUserProjectsError> {
    invoke_result("load_user_projects", LoadUserProjectsArgs { user }).await
}

pub async fn add_project(path: PathBuf) -> Result<(Project, ProjectSettings), String> {
    invoke_result("add_project", PathBufArgs { path }).await
}

pub async fn update_project(project: Project) -> DbResult {
    invoke_result("update_project", UpdateProjectArgs { project }).await
}

pub async fn get_project_path(project: ResourceId) -> Result<PathBuf, String> {
    invoke_result("get_project_path", ResourceIdArgs { rid: project }).await
}

pub async fn analyze(root: ResourceId) -> Result<(), AnalysisError> {
    invoke_result(
        "analyze",
        &AnalyzeArgs {
            root: root.clone(),
            max_tasks: None,
        },
    )
    .await
}

/// Arguments for [`load_user_projects`](thot_desktop_tauri::commands::project::load_user_projects).
#[derive(Serialize)]
pub struct LoadUserProjectsArgs {
    /// [`ResourceId`] of the user.
    pub user: ResourceId,
}

/// Arguments for creating a [`new_project`](thot_desktop_tauri::commands::project::new_project).
#[derive(Serialize)]
pub struct NewProjectArgs<'a> {
    /// Name of the project.
    pub name: &'a str,
}

/// Arguments for [`update_project`](thot_desktop_tauri::commands::project::update_project).
#[derive(Serialize)]
pub struct UpdateProjectArgs {
    /// Updated [`Project`]. `project.rid.id` is used as project to update.
    pub project: Project,
}

/// Arguments for `analyze`.
#[derive(Serialize)]
pub struct AnalyzeArgs {
    /// Root `Container`.
    pub root: ResourceId,

    /// Maximum number of allowed tasks.
    pub max_tasks: Option<usize>,
}
