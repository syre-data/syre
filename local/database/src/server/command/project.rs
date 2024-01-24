//! Handle `Project` related functionality.
use super::super::Database;
use crate::command::ProjectCommand;
use crate::error::server::LoadUserProjects as LoadUserProjectsError;
use crate::error::{Error, Result};
use serde_json::Value as JsValue;
use std::collections::HashMap;
use std::path::Path;
use std::result::Result as StdResult;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::Project as CoreProject;
use thot_core::types::{Creator, ResourceId, UserPermissions};
use thot_local::error::IoSerde as IoSerdeError;
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
                                let err: Result<CoreProject> = Err(err.into());
                                return serde_json::to_value(err).unwrap();
                            }
                        };

                        project
                    }
                };

                let project: Result<CoreProject> = Ok((**project).clone());
                serde_json::to_value(project).unwrap()
            }

            ProjectCommand::LoadWithSettings(path) => {
                // check if project is already loaded
                let project = match self.get_path_project(&path) {
                    Some(project) => project,
                    None => {
                        let project = match self.load_project(&path) {
                            Ok(project) => project,
                            Err(err) => {
                                let err: Result<CoreProject> = Err(err.into());
                                return serde_json::to_value(err).unwrap();
                            }
                        };

                        project
                    }
                };

                let project: Result<(CoreProject, ProjectSettings)> =
                    Ok(((**project).clone(), project.settings().clone()));

                serde_json::to_value(project).unwrap()
            }

            ProjectCommand::Add(path, user) => {
                let Ok(local_project) = self.load_project(&path) else {
                    let err: Result<CoreProject> =
                        Err(Error::SettingsError("could not load project".to_string()));
                    return serde_json::to_value(err).unwrap();
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
                        return serde_json::to_value(res).unwrap();
                    }
                }

                // add project to collection
                let mut projects = match Projects::load() {
                    Ok(projects) => projects,
                    Err(err) => {
                        let err = Error::SettingsError(format!("{err:?}"));
                        return serde_json::to_value(err).unwrap();
                    }
                };

                projects.insert(project.rid.clone(), path.to_path_buf());

                let res = projects.save();
                if res.is_err() {
                    let error = Error::SettingsError(format!("{res:?}"));
                    return serde_json::to_value(error).unwrap();
                };

                let project: Result<(CoreProject, ProjectSettings)> = Ok((project, settings));
                serde_json::to_value(project).unwrap()
            }

            ProjectCommand::LoadUser(user) => {
                let projects = self.load_user_projects(&user);
                serde_json::to_value(projects).unwrap()
            }

            ProjectCommand::Get(rid) => {
                let Some(project) = self.store.get_project(&rid) else {
                    let value: Option<CoreProject> = None;
                    return serde_json::to_value(value).unwrap();
                };

                let project = Some((**project).clone());
                serde_json::to_value(project).unwrap()
            }

            ProjectCommand::Update(update) => {
                let res = self.update_project(update);
                serde_json::to_value(res).unwrap()
            }

            ProjectCommand::GetPath(rid) => {
                let path = self.get_project_path(&rid);
                serde_json::to_value(path).unwrap()
            }

            ProjectCommand::ResourceRootPath(path) => {
                let path = project_resource_root_path(&path);
                serde_json::to_value(path).unwrap()
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
    ///
    /// # Side effects
    /// + Watches the project folder.
    pub fn load_project(&mut self, path: &Path) -> StdResult<&LocalProject, IoSerdeError> {
        let project = LocalProject::load_from(path)?;
        self.store.insert_project(project)?;
        self.watch_path(path);
        let project = self.get_path_project(&path).unwrap();
        return Ok(project);
    }

    fn get_path_project(&self, path: &Path) -> Option<&LocalProject> {
        let Ok(Some(project)) = self.store.get_path_project_canonical(&path) else {
            return None;
        };

        self.store.get_project(project)
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
    ) -> StdResult<Vec<(CoreProject, ProjectSettings)>, LoadUserProjectsError> {
        let project_manifest = match Projects::load_or_default() {
            Ok(project_manifest) => project_manifest,
            Err(err) => return Err(LoadUserProjectsError::LoadProjectsManifest(err)),
        };

        // load projects
        let mut projects = Vec::new();
        let mut errors = HashMap::new();
        for (pid, path) in project_manifest.iter() {
            match self.store.get_project(&pid) {
                Some(project) => {
                    if user_has_project(user, &project) {
                        projects.push(((*project).clone(), project.settings().clone()));
                    }
                }

                None => match self.load_project(&path) {
                    Ok(_) => {}
                    Err(err) => {
                        errors.insert(path.clone(), err);
                    }
                },
            };
        }

        // TODO Unload unused projects.
        if errors.is_empty() {
            Ok(projects)
        } else {
            Err(LoadUserProjectsError::LoadProjects { projects, errors })
        }
    }

    fn update_project(&mut self, update: CoreProject) -> Result {
        let Some(project) = self.store.get_project_mut(&update.rid) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Script` does not exist",
            ))
            .into());
        };

        **project = update;
        project.save()?;
        Ok(())
    }

    fn update_project_settings(&mut self, rid: &ResourceId, settings: ProjectSettings) -> Result {
        let Some(project) = self.store.get_project_mut(rid) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Script` does not exist",
            ))
            .into());
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
