//! Handle [`syre::Script`](ScriptEvent) events.
use super::event::app::Script as ScriptEvent;
use crate::event::{Script as ScriptUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::fs;
use syre_local::error::{Error as LocalError, Project as ProjectError};
use syre_local::project::project;
use syre_local::project::resources::Script as LocalScript;

impl Database {
    pub fn handle_app_event_script(&mut self, event: &ScriptEvent) -> Result {
        match event {
            ScriptEvent::Created(path) => {
                let Some(project_path) = project::project_root_path(&path) else {
                    return Err(
                        LocalError::Project(ProjectError::PathNotInProject(path.clone())).into(),
                    );
                };

                let project_path = fs::canonicalize(project_path)?;
                let project = self
                    .store
                    .get_path_project_canonical(&project_path)
                    .unwrap()
                    .unwrap()
                    .clone();

                let project = self.store.get_project(&project).unwrap();
                let pid = project.rid.clone();
                let script_path = path
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let script = LocalScript::new(script_path)?;
                self.store.insert_script(pid.clone(), script.clone())?;

                self.publish_update(&Update::Project {
                    project: pid,
                    update: ScriptUpdate::Created(script).into(),
                })?;

                Ok(())
            }

            ScriptEvent::Removed(script) => {
                let project = self.store.get_script_project(&script).unwrap().clone();
                self.store.remove_project_script(&project, &script)?;

                self.publish_update(&Update::Project {
                    project,
                    update: ScriptUpdate::Removed(script.clone()).into(),
                })?;

                Ok(())
            }

            ScriptEvent::Moved { script, path } => {
                let from_project = self.store.get_script_project(&script).unwrap().clone();
                let to_project = project::project_root_path(&path).unwrap();
                let to_project = self
                    .store
                    .get_path_project_canonical(&to_project)
                    .unwrap()
                    .unwrap()
                    .clone();

                if to_project == from_project {
                    let project = self.store.get_project(&to_project).unwrap();
                    let script_path = path
                        .strip_prefix(project.analysis_root_path().unwrap())
                        .unwrap()
                        .to_owned();

                    let scripts = self.store.get_project_scripts_mut(&from_project).unwrap();
                    let script = scripts.get_script_mut(&script).unwrap();
                    let sid = script.rid.clone();
                    script.path = script_path.clone();
                    scripts.save()?;

                    self.publish_update(&Update::Project {
                        project: from_project,
                        update: ScriptUpdate::Moved {
                            script: sid,
                            path: script_path,
                        }
                        .into(),
                    })?;
                } else {
                    let script = self
                        .store
                        .remove_project_script(&from_project, &script)?
                        .unwrap();

                    self.publish_update(&Update::Project {
                        project: from_project,
                        update: ScriptUpdate::Removed(script.rid.clone()).into(),
                    })?;

                    self.store
                        .insert_script(to_project.clone(), script.clone())
                        .unwrap();

                    self.publish_update(&Update::Project {
                        project: to_project.clone(),
                        update: ScriptUpdate::Created(script).into(),
                    })?;
                }

                Ok(())
            }
        }
    }
}
