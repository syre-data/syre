//! Resources for [`project commands`](thot_desktop_tauri::commands::project).
use serde::Serialize;
use thot_core::project::Project;
use thot_core::types::ResourceId;

/// Arguments for [`load_user_projects`](thot_desktop_tauri::commands::project::load_user_projects).
#[derive(Serialize)]
pub struct LoadUserProjectsArgs {
    /// [`ResourceId`] of the user.
    pub user: ResourceId,
}

/// Arguments for [`get_project_path`](thot_desktop_tauri::commands::project::get_project_path).
#[derive(Serialize)]
pub struct GetProjectPathArgs {
    /// Id of the [`Project`](thot_core::project::Project).
    pub id: ResourceId,
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

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
