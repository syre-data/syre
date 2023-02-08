//! Hooks related to the [`ProjectsState`](crate::app::ProjectsState).
pub mod active_project;
pub mod open_projects;
pub mod project;
pub mod project_scripts;
pub mod user_projects;

// Re-exports
pub use active_project::use_active_project;
pub use open_projects::use_open_projects;
pub use project::use_project;
pub use project_scripts::use_project_scripts;
pub use user_projects::use_user_projects;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
