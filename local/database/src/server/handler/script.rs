//! Handle `Script` related functionality.
use super::super::Database;
use crate::command::ScriptCommand;
use crate::Result;
use serde_json::Value as JsValue;
use settings_manager::{LocalSettings, SystemSettings};
use std::path::PathBuf;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::{Project as CoreProject, Script as CoreScript};
use thot_core::types::{ResourceId, ResourcePath};
use thot_local::project::resources::{
    Project as LocalProject, Script as LocalScript, Scripts as ProjectScripts,
};
use thot_local::system::collections::Projects;

impl Database {
    pub fn handle_command_script(&mut self, cmd: ScriptCommand) -> JsValue {
        match cmd {
            ScriptCommand::Get(script) => {
                let script = self.store.get_script(&script);
                serde_json::to_value(script.clone()).expect("could not convert `Script` to JsValue")
            }

            ScriptCommand::Add(project, script) => {
                let script = self.add_script(project, script);
                serde_json::to_value(script).expect("could not convert `Script` to JsValue")
            }

            ScriptCommand::Remove(project, script) => {
                let res = self.remove_script(&project, &script);
                serde_json::to_value(res).expect("could not convert to JsValue")
            }

            ScriptCommand::Update(script) => {
                let res = self.update_script(script);
                serde_json::to_value(res).expect("could not convert result to JsValue")
            }

            ScriptCommand::LoadProject(project) => {
                let scripts = self.load_project_scripts(project);
                serde_json::to_value(scripts).expect("could not convert result to JsValue")
            }

            ScriptCommand::GetProject(script) => {
                let project = self.get_script_project(&script);
                let Some(project) = project else {
                    let val: Option<CoreProject> = None;
                    return serde_json::to_value(val).expect("could not convert `CoreProject` to JsValue")
                };

                let project: Option<CoreProject> = Some((**project).clone());
                serde_json::to_value(project).expect("could not convert `CoreProject` to JsValue")
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
            let scripts = (*scripts).clone().into_values().collect();
            return Ok(scripts);
        }

        let projects = Projects::load_or_default()?;
        let Some(project) = projects.get(&rid).clone() else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` does not exist")).into());
        };

        let scripts = ProjectScripts::load_or_default(&project.path)?;
        let script_vals = (*scripts).clone().into_values().collect();
        self.store.insert_project_scripts(rid, scripts);

        Ok(script_vals)
    }

    /// Adds a `Script` to a `Project`.
    fn add_script(&mut self, project: ResourceId, script: PathBuf) -> Result<CoreScript> {
        let script = LocalScript::new(ResourcePath::new(script)?)?;
        self.store.insert_script(project, script.clone())?;

        Ok(script)
    }

    /// Remove `Script` from `Project`.
    fn remove_script(&mut self, pid: &ResourceId, script: &ResourceId) -> Result {
        self.store.remove_project_script(pid, script)?;
        Ok(())
    }

    /// Update a `Script`.
    fn update_script(&mut self, script: CoreScript) -> Result {
        let Some(project) = self.store.get_script_project(&script.rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Script` does not exist")).into());
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
