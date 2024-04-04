//! Handle `Project` related functionality.
use super::super::Database;
use crate::command::ProjectCommand;
use crate::error::server::{LoadUserProjects as LoadUserProjectsError, Update as UpdateError};
use crate::error::Result;
use crate::server::store::data_store::data_store::project::Record;
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
                serde_json::to_value(self.handle_project_load(path.as_path())).unwrap()
            }

            ProjectCommand::LoadWithSettings(path) => {
                serde_json::to_value(self.handle_project_load_with_settings(path.as_path()))
                    .unwrap()
            }

            ProjectCommand::LoadUser(user) => {
                let projects = self.handle_project_load_user(&user);
                serde_json::to_value(projects).unwrap()
            }

            ProjectCommand::Get(project) => {
                serde_json::to_value(self.handle_project_get(&project)).unwrap()
            }

            ProjectCommand::Update(update) => {
                let res = self.handle_project_update(update);
                serde_json::to_value(res).unwrap()
            }

            ProjectCommand::GetPath(rid) => {
                let path = self.handle_project_path(&rid);
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

    fn handle_project_get(&self, project: &ResourceId) -> Option<CoreProject> {
        match self.object_store.get_project(project) {
            None => None,
            Some(project) => Some((**project).clone()),
        }
    }

    fn handle_project_load(&mut self, path: &Path) -> StdResult<CoreProject, IoSerdeError> {
        let project = match self.get_path_project(path) {
            Some(project) => project,
            None => self.load_project(path)?,
        };

        Ok((*project).clone())
    }

    fn handle_project_load_with_settings(
        &mut self,
        path: &Path,
    ) -> StdResult<(CoreProject, ProjectSettings), IoSerdeError> {
        let project = match self.get_path_project(&path) {
            Some(project) => project,
            None => self.load_project(&path)?,
        };

        Ok(((*project).clone(), project.settings().clone()))
    }

    /// Loads a single [`Project`](LocalProject) from settings.
    ///
    /// # Returns
    /// Reference to the loaded [`Project`](LocalProject).
    ///
    /// # Side effects
    /// + Watches the project folder.
    pub fn load_project(&mut self, path: &Path) -> StdResult<&LocalProject, IoSerdeError> {
        let project = LocalProject::load_from(path)?;
        self.object_store.insert_project(project)?;
        self.watch_path(path);

        let project = self.get_path_project(&path).unwrap();
        if let Err(err) = self.data_store.project().create(
            project.rid.clone(),
            Record::new(
                project.name.clone(),
                project.description.clone(),
                project.base_path().to_path_buf(),
            ),
        ) {
            tracing::error!(?err);
        }

        Ok(project)
    }

    fn get_path_project(&self, path: &Path) -> Option<&LocalProject> {
        let Ok(Some(project)) = self.object_store.get_path_project_canonical(&path) else {
            return None;
        };

        self.object_store.get_project(project)
    }

    fn handle_project_path(&self, rid: &ResourceId) -> Option<&Path> {
        let Some(project) = self.object_store.get_project(rid) else {
            return None;
        };

        Some(project.base_path())
    }

    fn handle_project_load_user(
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
        for path in project_manifest.iter() {
            match self.object_store.get_path_project(path) {
                Some(project) => {
                    let project = self.object_store.get_project(project).unwrap();
                    if user_has_project(user, project) {
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

    fn handle_project_update(&mut self, update: CoreProject) -> StdResult<(), UpdateError> {
        let Some(project) = self.object_store.get_project_mut(&update.rid) else {
            return Err(UpdateError::ResourceNotFound);
        };

        **project = update;
        project.save()?;

        if let Err(err) = self.data_store.project().update(
            project.rid.clone(),
            Record::new(
                project.name.clone(),
                project.description.clone(),
                project.base_path().to_path_buf(),
            ),
        ) {
            tracing::error!(?err);
        }

        Ok(())
    }

    fn update_project_settings(&mut self, rid: &ResourceId, settings: ProjectSettings) -> Result {
        let Some(project) = self.object_store.get_project_mut(rid) else {
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
