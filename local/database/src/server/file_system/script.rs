//! Handle [`thot::Script`](ScriptEvent) events.
use super::event::thot::Script as ScriptEvent;
use crate::event::{Script as ScriptUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::fs;
use thot_core::types::ResourcePath;
use thot_local::project::project;
use thot_local::project::resources::Script as LocalScript;

impl Database {
    pub fn handle_thot_event_script(&mut self, event: ScriptEvent) -> Result {
        match event {
            ScriptEvent::Created(path) => {
                let project_path = fs::canonicalize(project::project_root_path(&path)?).unwrap();
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

                let path = ResourcePath::new(script_path.to_path_buf())?;
                let script = LocalScript::new(path)?;
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
                    update: ScriptUpdate::Removed(script).into(),
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
                    let script = scripts.get_mut(&script).unwrap();
                    let sid = script.rid.clone();
                    let script_path = ResourcePath::new(script_path.clone())?;
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
