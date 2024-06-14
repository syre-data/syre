//! Handle `Script` related functionality.
use super::super::Database;
use crate::command::AnalysisCommand;
use crate::{Error, Result};
use serde_json::Value as JsValue;
use std::path::PathBuf;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::{ExcelTemplate, Project as CoreProject, Script as CoreScript};
use syre_core::types::ResourceId;
use syre_local::project::resources::{
    Analyses as ProjectScripts, Project as LocalProject, Script as LocalScript,
};
use syre_local::types::{AnalysisKind, AnalysisStore};

impl Database {
    pub fn handle_command_analysis(&mut self, cmd: AnalysisCommand) -> JsValue {
        match cmd {
            AnalysisCommand::Get(script) => {
                let script = self.object_store.get_analysis(&script);
                serde_json::to_value(&script).unwrap()
            }

            AnalysisCommand::AddScript(project, script) => {
                let script = self.add_script(project, script);
                serde_json::to_value(script).unwrap()
            }

            AnalysisCommand::AddExcelTemplate { project, template } => {
                let res = self.object_store.insert_excel_template(project, template);
                serde_json::to_value(res).unwrap()
            }

            AnalysisCommand::UpdateExcelTemplate(template) => {
                let res = self.update_excel_template(template);
                serde_json::to_value(res).unwrap()
            }

            AnalysisCommand::Remove { project, script } => {
                let res = self.remove_analysis(&project, &script);
                serde_json::to_value(res).unwrap()
            }

            AnalysisCommand::UpdateScript(script) => {
                let res = self.update_script(script);
                serde_json::to_value(res).unwrap()
            }

            AnalysisCommand::LoadProject(project) => {
                let scripts = self.load_project_scripts(project);
                serde_json::to_value(scripts).unwrap()
            }

            AnalysisCommand::GetProject(script) => {
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
    fn load_project_scripts(&mut self, rid: ResourceId) -> Result<AnalysisStore> {
        if let Some(scripts) = self.object_store.get_project_scripts(&rid) {
            // project scripts already loaded
            return Ok((**scripts).clone());
        }

        let Some(project) = self.object_store.get_project(&rid) else {
            return Err(CoreError::Resource(ResourceError::DoesNotExist(
                "project is not loaded".to_string(),
            ))
            .into());
        };

        let scripts = ProjectScripts::load_from(project.base_path())?;
        let script_vals = (*scripts).clone();
        self.object_store.insert_project_scripts(rid, scripts);
        Ok(script_vals)
    }

    /// Adds a `Script` to a `Project`.
    fn add_script(&mut self, project: ResourceId, script: PathBuf) -> Result<CoreScript> {
        let script = LocalScript::new(script)?;
        self.object_store.insert_script(project, script.clone())?;

        Ok(script)
    }

    /// Remove an analysis from `Project`.
    fn remove_analysis(&mut self, pid: &ResourceId, analysis: &ResourceId) -> Result {
        let analysis = self.object_store.remove_project_script(pid, analysis)?;

        if let Some(analysis) = analysis {
            let analysis_path = match analysis {
                AnalysisKind::Script(script) => script.path,
                AnalysisKind::ExcelTemplate(template) => template.template.path,
            };

            let path = if analysis_path.is_absolute() {
                analysis_path.clone()
            } else if analysis_path.is_relative() {
                let Some(project) = self.object_store.get_project(&pid) else {
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
                .join(analysis_path)
            } else {
                todo!("unhandled path type");
            };

            // TODO: Ensure no other scripts or tempaltes rely on the file
            //      before removing.
            trash::delete(path)?;
        }

        Ok(())
    }

    /// Update a `Script`.
    fn update_script(&mut self, script: CoreScript) -> Result {
        let Some(project) = self.object_store.get_script_project(&script.rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Script` does not exist",
            ))
            .into());
        };

        self.object_store.insert_script(project.clone(), script)?;
        Ok(())
    }

    fn update_excel_template(&mut self, template: ExcelTemplate) -> Result {
        let Some(project) = self.object_store.get_script_project(&template.rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`ExcelTemplate` does not exist",
            ))
            .into());
        };

        self.object_store
            .insert_excel_template(project.clone(), template)?;

        Ok(())
    }

    /// Get the `Project` of a `Script`.
    fn get_script_project(&self, script: &ResourceId) -> Option<&LocalProject> {
        let Some(project) = self.object_store.get_script_project(script) else {
            return None;
        };

        self.object_store.get_project(project)
    }
}

#[cfg(test)]
#[path = "./analysis_test.rs"]
mod analysis_test;
