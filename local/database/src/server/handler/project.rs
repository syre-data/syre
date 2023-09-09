//! Handle `Project` related functionality.
use super::super::Database;
use crate::command::ProjectCommand;
use crate::error::{Error, Result};
use serde_json::Value as JsValue;
use std::path::Path;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::Project as CoreProject;
use thot_core::types::{Creator, ResourceId, UserPermissions};
use thot_local::project::project::project_resource_root_path;
use thot_local::project::resources::project::Project as LocalProject;
use thot_local::system::collections::projects::Projects;
use thot_local::types::ProjectSettings;

impl Database {
    /// Directs the command to the correct handler.
    pub fn handle_command_project(&mut self, cmd: ProjectCommand) -> JsValue {
        match cmd {
            ProjectCommand::Load(path) => {
                // check if project is already loaded
                let project = match self.get_path_project(&path) {
                    Some(project) => project,
                    None => {
                        let project = match self.load_project(&path) {
                            Ok(project) => project,
                            Err(err) => {
                                let err: Result<CoreProject> = Err(err);
                                return serde_json::to_value(err)
                                    .expect("could not convert `Project` to JsValue");
                            }
                        };

                        project
                    }
                };

                let project: Result<CoreProject> = Ok((**project).clone());
                serde_json::to_value(project).expect("could not convert `Project` to JsValue")
            }

            ProjectCommand::LoadWithSettings(path) => {
                // check if project is already loaded
                let project = match self.get_path_project(&path) {
                    Some(project) => project,
                    None => {
                        let project = match self.load_project(&path) {
                            Ok(project) => project,
                            Err(err) => {
                                let err: Result<CoreProject> = Err(err);
                                return serde_json::to_value(err)
                                    .expect("could not convert `Project` to JsValue");
                            }
                        };

                        project
                    }
                };

                let project: Result<(CoreProject, ProjectSettings)> =
                    Ok(((**project).clone(), project.settings().clone()));

                serde_json::to_value(project).expect("could not convert `Project` to JsValue")
            }

            ProjectCommand::Add(path, user) => {
                let Ok(local_project) = self.load_project(&path) else {
                    let err: Result<CoreProject> = Err(Error::SettingsError("could not load project".to_string()));
                    return serde_json::to_value(err).expect("could not convert error to JsValue");
                };

                let project = (*local_project).clone();
                let settings = local_project.settings().clone();
                if !user_has_project(&user, &local_project) {
                    let mut settings = settings.clone();
                    let permissions = UserPermissions {
                        read: true,
                        write: true,
                        execute: true,
                    };

                    settings.permissions.insert(user, permissions);
                    let res = self.update_project_settings(&project.rid, settings);
                    if res.is_err() {
                        return serde_json::to_value(res)
                            .expect("could not convert error to JsValue");
                    }
                }

                // add project to collection
                let mut projects = match Projects::load() {
                    Ok(projects) => projects,
                    Err(err) => {
                        let err = Error::SettingsError(format!("{err:?}"));
                        return serde_json::to_value(err)
                            .expect("could not convert error to JsValue");
                    }
                };

                projects.insert(project.rid.clone(), path.to_path_buf());

                let res = projects.save();
                if res.is_err() {
                    let error = Error::SettingsError(format!("{res:?}"));
                    return serde_json::to_value(error)
                        .expect("could not convert error to JsValue");
                };

                let project: Result<(CoreProject, ProjectSettings)> = Ok((project, settings));

                serde_json::to_value(project).expect("could not convert `Project` to JsValue")
            }

            ProjectCommand::LoadUser(user) => {
                let projects = self.load_user_projects(&user);
                serde_json::to_value(projects).expect("could not convert `Project`s to JsValue")
            }

            ProjectCommand::Get(rid) => {
                let Some(project) = self.store.get_project(&rid) else {
                    let value: Option<CoreProject> = None;
                    return serde_json::to_value(value).expect("could not convert `None` to JsValue")
                };

                let project = Some((**project).clone());
                serde_json::to_value(project).expect("could not convert `Project` to JsValue")
            }

            ProjectCommand::Update(update) => {
                let res = self.update_project(update);
                serde_json::to_value(res)
                    .expect("could not convert `update_project` result to JsValue")
            }

            ProjectCommand::GetPath(rid) => {
                let path = self.get_project_path(&rid);
                serde_json::to_value(path).expect("could not convert `PathBuf` to JsValue")
            }

            ProjectCommand::ResourceRootPath(path) => {
                let path = project_resource_root_path(&path);
                serde_json::to_value(path).expect("could not convert `PathBuf` to JsValue")
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
    pub fn load_project(&mut self, path: &Path) -> Result<&LocalProject> {
        // load project
        let project = LocalProject::load_from(path)?;
        let _o_project = self.store.insert_project(project)?;
        if let Some(project) = self.get_path_project(&path) {
            return Ok(project);
        }

        Err(Error::DatabaseError(
            "could not store `Project`".to_string(),
        ))
    }

    fn get_path_project(&self, path: &Path) -> Option<&LocalProject> {
        if let Some(pid) = self.store.get_path_project(&path) {
            if let Some(project) = self.store.get_project(pid) {
                // already loaded
                return Some(project);
            }
        }

        None
    }

    fn get_project_path(&self, rid: &ResourceId) -> Option<&Path> {
        let Some(project) = self.store.get_project(rid) else {
            return None;
        };

        Some(project.base_path())
    }

    fn load_user_projects(
        &mut self,
        user: &ResourceId,
    ) -> Result<Vec<(CoreProject, ProjectSettings)>> {
        let projects_info = Projects::load_or_default()?;

        // load projects
        let mut projects = Vec::new();
        for (pid, path) in projects_info.iter() {
            let project = match self.store.get_project(&pid) {
                Some(project) => project,
                None => self.load_project(&path)?,
            };

            if user_has_project(user, &project) {
                projects.push(((*project).clone(), project.settings().clone()));
            }
        }

        // TODO Unload unused projects.
        Ok(projects)
    }

    fn update_project(&mut self, update: CoreProject) -> Result {
        let Some(project) = self.store.get_project_mut(&update.rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Script` does not exist")).into());
        };

        **project = update;
        project.save()?;
        Ok(())
    }

    fn update_project_settings(&mut self, rid: &ResourceId, settings: ProjectSettings) -> Result {
        let Some(project) = self.store.get_project_mut(rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Script` does not exist")).into());
        };

        *project.settings_mut() = settings;
        project.save()?;
        Ok(())
    }
}

// ************************
// *** helper functions ***
// ************************

/// Returns if the user has any permissions on the project.
fn user_has_project(user: &ResourceId, project: &LocalProject) -> bool {
    let creator = Creator::User(Some(user.clone().into()));
    project.creator == creator || project.settings().permissions.contains_key(user)
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
