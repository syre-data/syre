//! Handle [`syre::Script`](ScriptEvent) events.
use super::event::app::Script as ScriptEvent;
use crate::event::{Script as ScriptUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::fs;
use syre_local::error::{Error as LocalError, Project as ProjectError};
use syre_local::project::project;
use syre_local::project::resources::Script as LocalScript;
use syre_local::types::AnalysisKind;
use uuid::Uuid;

impl Database {
    pub fn handle_app_event_script(
        &mut self,
        event: &ScriptEvent,
        event_id: &Uuid,
    ) -> Result<Vec<Update>> {
        match event {
            ScriptEvent::Created(path) => {
                let Some(project_path) = project::project_root_path(&path) else {
                    return Err(
                        LocalError::Project(ProjectError::PathNotInProject(path.clone())).into(),
                    );
                };

                let project_path = fs::canonicalize(project_path)?;
                let project = self
                    .object_store
                    .get_path_project_canonical(&project_path)
                    .unwrap()
                    .unwrap()
                    .clone();

                let project = self.object_store.get_project(&project).unwrap();
                let pid = project.rid.clone();
                let script_path = path
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let script = LocalScript::new(script_path)?;
                self.object_store
                    .insert_script(pid.clone(), script.clone())?;

                Ok(vec![Update::project(
                    pid,
                    ScriptUpdate::Created(script).into(),
                    event_id.clone(),
                )])
            }

            ScriptEvent::Removed(script) => {
                let project = self
                    .object_store
                    .get_script_project(&script)
                    .unwrap()
                    .clone();
                self.object_store.remove_project_script(&project, &script)?;

                Ok(vec![Update::project(
                    project,
                    ScriptUpdate::Removed(script.clone()).into(),
                    event_id.clone(),
                )])
            }

            ScriptEvent::Moved { script, path } => {
                let from_project = self
                    .object_store
                    .get_script_project(&script)
                    .unwrap()
                    .clone();
                let to_project = project::project_root_path(&path).unwrap();
                let to_project = self
                    .object_store
                    .get_path_project_canonical(&to_project)
                    .unwrap()
                    .unwrap()
                    .clone();

                if to_project == from_project {
                    let project = self.object_store.get_project(&to_project).unwrap();
                    let script_path = path
                        .strip_prefix(project.analysis_root_path().unwrap())
                        .unwrap()
                        .to_owned();

                    let analyses = self
                        .object_store
                        .get_project_scripts_mut(&from_project)
                        .unwrap();
                    let AnalysisKind::Script(script) = analyses.get_mut(&script).unwrap() else {
                        todo!("handle other analysis kinds");
                    };

                    let sid = script.rid.clone();
                    script.path = script_path.clone();
                    analyses.save()?;

                    return Ok(vec![Update::project(
                        from_project,
                        ScriptUpdate::Moved {
                            script: sid,
                            path: script_path,
                        }
                        .into(),
                        event_id.clone(),
                    )]);
                } else {
                    let analysis = match self
                        .object_store
                        .remove_project_script(&from_project, &script)?
                        .unwrap()
                    {
                        AnalysisKind::Script(script) => script,
                        AnalysisKind::ExcelTemplate(template) => todo!("handle template"),
                    };

                    let mut updates = vec![Update::project(
                        from_project,
                        ScriptUpdate::Removed(analysis.rid.clone()).into(),
                        event_id.clone(),
                    )];

                    if let Err(err) = self
                        .object_store
                        .insert_script(to_project.clone(), analysis.clone())
                    {
                        tracing::error!("could not insert script: {err:?}");
                        return Ok(updates);
                    }

                    updates.push(Update::project(
                        to_project.clone(),
                        ScriptUpdate::Created(analysis).into(),
                        event_id.clone(),
                    ));

                    return Ok(updates);
                }
            }
        }
    }
}
