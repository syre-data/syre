//! Handle `Script` related functionality.
use super::super::Database;
use crate::command::ScriptCommand;
use crate::{Error, Result};
use serde_json::Value as JsValue;
use std::path::PathBuf;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::{Project as CoreProject, Script as CoreScript};
use syre_core::types::ResourceId;
use syre_local::project::resources::{
    Project as LocalProject, Script as LocalScript, Scripts as ProjectScripts,
};

impl Database {
    pub fn handle_command_script(&mut self, cmd: ScriptCommand) -> JsValue {
        match cmd {
            ScriptCommand::Get(script) => {
                let script = self.store.get_script(&script);
                serde_json::to_value(script.clone()).unwrap()
            }

            ScriptCommand::Add(project, script) => {
                let script = self.add_script(project, script);
                serde_json::to_value(script).unwrap()
            }

            ScriptCommand::Remove(project, script) => {
                let res = self.remove_script(&project, &script);
                serde_json::to_value(res).unwrap()
            }

            ScriptCommand::Update(script) => {
                let res = self.update_script(script);
                serde_json::to_value(res).unwrap()
            }

            ScriptCommand::LoadProject(project) => {
                let scripts = self.load_project_scripts(project);
                serde_json::to_value(scripts).unwrap()
            }

            ScriptCommand::GetProject(script) => {
                let project = self.get_script_project(&script);
                let Some(project) = project else {
                    let val: Option<CoreProject> = None;
                    return serde_json::to_value(val).unwrap();
                };

                let project: Option<CoreProject> = Some((**project).clone());
                serde_json::to_value(project).unwrap()
            }
        }
    }

    /// Loads a `Project`'s `Scripts`.
    ///
    /// # Arguments
    /// 1. `Project`'s id.
    fn load_project_scripts(&mut self, rid: ResourceId) -> Result<Vec<CoreScript>> {
        if let Some(scripts) = self.store.get_project_scripts(&rid) {
            // project scripts already loaded
            return Ok(scripts.values().map(|script| script.clone()).collect());
        }

        let Some(project) = self.store.get_project(&rid) else {
            return Err(CoreError::Resource(ResourceError::DoesNotExist(
                "project is not loaded".to_string(),
            ))
            .into());
        };

        let scripts = ProjectScripts::load_from(project.base_path())?;
        let script_vals = (**scripts).clone().into_values().collect();
        self.store.insert_project_scripts(rid, scripts);
        Ok(script_vals)
    }

    /// Adds a `Script` to a `Project`.
    fn add_script(&mut self, project: ResourceId, script: PathBuf) -> Result<CoreScript> {
        let script = LocalScript::new(script)?;
        self.store.insert_script(project, script.clone())?;

        Ok(script)
    }

    /// Remove `Script` from `Project`.
    fn remove_script(&mut self, pid: &ResourceId, script: &ResourceId) -> Result {
        let script = self.store.remove_project_script(pid, script)?;

        if let Some(script) = script {
            let path = if script.path.is_absolute() {
                script.path.clone()
            } else if script.path.is_relative() {
                let Some(project) = self.store.get_project(&pid) else {
                    return Err(Error::Database(String::from(
                        "could not get `Project` path",
                    )));
                };

                let path = project.base_path();
                path.join(
                    project
                        .analysis_root
                        .as_ref()
                        .expect("`Project`'s analysis root not set")
                        .clone(),
                )
                .join(script.path)
            } else {
                todo!("unhandled path type");
            };

            trash::delete(path)?;
        }

        Ok(())
    }

    /// Update a `Script`.
    fn update_script(&mut self, script: CoreScript) -> Result {
        let Some(project) = self.store.get_script_project(&script.rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Script` does not exist",
            ))
            .into());
        };

        self.store.insert_script(project.clone(), script)?;
        Ok(())
    }

    /// Get the `Project` of a `Script`.
    fn get_script_project(&self, script: &ResourceId) -> Option<&LocalProject> {
        let Some(project) = self.store.get_script_project(script) else {
            return None;
        };

        self.store.get_project(project)
    }
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
