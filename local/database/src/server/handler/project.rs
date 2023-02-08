//! Handle `Project` related functionality.
use super::super::Database;
use crate::command::ProjectCommand;
use crate::Result;
use serde_json::Value as JsValue;
use settings_manager::{LocalSettings, SystemSettings};
use std::path::PathBuf;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::Project as CoreProject;
use thot_core::types::{Creator, ResourceId};
use thot_local::project::resources::Project as LocalProject;
use thot_local::system::collections;

impl Database {
    /// Directs the command to the correct handler.
    pub fn handle_command_project(&mut self, cmd: ProjectCommand) -> JsValue {
        match cmd {
            ProjectCommand::LoadProject(path) => {
                let project = self.load_project(path);
                serde_json::to_value(project).expect("could not convert `Project` to JsValue")
            }

            ProjectCommand::LoadUserProjects(user) => {
                let projects = self.load_user_projects(user);
                serde_json::to_value(projects).expect("could not convert `Project`s to JsValue")
            }

            ProjectCommand::GetProject(rid) => {
                let project = self.get_project(rid);
                serde_json::to_value(project).expect("could not convert `Project` to JsValue")
            }

            ProjectCommand::UpdateProject(update) => {
                let res = self.update_project(update);
                serde_json::to_value(res)
                    .expect("could not convert `update_project` result to JsValue")
            }
        }
    }
    // *****************
    // *** functions ***
    // *****************

    /// Loads a single [`Project`](LocalProject) from settings.
    ///
    /// # Returns
    /// Reference to the loaded [`Project`](LocalProject).
    pub fn load_project(&mut self, path: PathBuf) -> Result<CoreProject> {
        if let Some(pid) = self.store.get_path_project(&path) {
            if let Some(project) = self.store.get_project(pid) {
                // already loaded
                return Ok((**project).clone());
            }
        }

        let project = LocalProject::load(&path)?;
        let project_val = (*project).clone();
        let _o_project = self.store.insert_project(project);
        Ok(project_val)
    }

    fn load_user_projects(&mut self, user: ResourceId) -> Result<Vec<CoreProject>> {
        // get project info
        let projects_info = collections::Projects::load()?;
        let user = Creator::User(Some(user.into()));

        // load projects
        let mut projects = Vec::new();
        for (_pid, project_info) in projects_info.clone().into_iter() {
            let project = self
                .load_project(project_info.path.clone())
                .expect("could not load `Project`");

            // @todo: Filter on permissions, not only creator.
            if project.creator == user {
                projects.push(project);
            }
        }

        // @todo: Unload unused projects.
        Ok(projects)
    }

    fn get_project(&self, rid: ResourceId) -> Option<CoreProject> {
        let Some(project) = self.store.get_project(&rid) else {
            return None;
        };

        Some((**project).clone())
    }

    fn update_project(&mut self, update: CoreProject) -> Result {
        let Some(project) = self.store.get_project_mut(&update.rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Script` does not exist".to_string())).into());
        };

        **project = update;
        project.save()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
