//! Handle `Project` related functionality.
use super::super::Database;
use crate::command::ProjectCommand;
use crate::error::server::LoadUserProjects as LoadUserProjectsError;
use crate::error::Result;
use serde_json::Value as JsValue;
use std::collections::HashMap;
use std::path::Path;
use std::result::Result as StdResult;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::Project as CoreProject;
use syre_core::types::{Creator, ResourceId};
use syre_local::error::IoSerde as IoSerdeError;
use syre_local::project::project::project_resource_root_path;
use syre_local::project::resources::project::Project as LocalProject;
use syre_local::system::collections::project_manifest::ProjectManifest;
use syre_local::types::ProjectSettings;

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
        let project_manifest = match ProjectManifest::load_or_default() {
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
                        projects.push((project.inner().clone(), project.settings().clone()));
                    }
                }

                None => match self.load_project(&path) {
                    Ok(project) => {
                        projects.push((project.inner().clone(), project.settings().clone()));
                    }

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
            return Err(CoreError::Resource(ResourceError::does_not_exist(
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
            return Err(CoreError::Resource(ResourceError::does_not_exist(
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
