use super::event::app::Project as ProjectEvent;
use crate::error::server::LoadUserProjects as LoadUserProjectsError;
use crate::event::{Project as ProjectUpdate, Update};
use crate::server::types::ProjectResources;
use crate::server::Database;
use crate::{Error, Result};
use syre_local::system::collections::project_manifest::ProjectManifest;

impl Database {
    pub fn handle_app_event_project(&mut self, event: &ProjectEvent) -> Result {
        match event {
            ProjectEvent::Moved { project, path } => {
                match self.store.update_project_path(&project, path.clone()) {
                    Ok(from) => {
                        let mut project_manifest = match ProjectManifest::load() {
                            Ok(project_manifest) => project_manifest,
                            Err(err) => {
                                return Err(Error::Database(format!(
                                    "{:?}",
                                    LoadUserProjectsError::LoadProjectsManifest(err)
                                )))
                            }
                        };

                        project_manifest.push(path.clone());
                        project_manifest.save()?;

                        self.unwatch_path(from);
                        self.watch_path(path);

                        self.publish_update(&Update::Project {
                            project: project.clone(),
                            update: ProjectUpdate::Moved(path.clone()),
                        })?;

                        return Ok(());
                    }

                    Err(err) => {
                        tracing::debug!(?err);
                        panic!("{err:?}");
                    }
                }
            }

            ProjectEvent::Removed(project) => {
                let ProjectResources { project, graph: _ } = self.store.remove_project(&project);
                if let Some(project) = project {
                    let mut project_manifest = match ProjectManifest::load() {
                        Ok(project_manifest) => project_manifest,
                        Err(err) => {
                            return Err(Error::Database(format!(
                                "{:?}",
                                LoadUserProjectsError::LoadProjectsManifest(err)
                            )))
                        }
                    };

                    project_manifest.remove(project.base_path());
                    project_manifest.save()?;
                    self.unwatch_path(project.base_path());

                    self.publish_update(&Update::Project {
                        project: project.rid.clone(),
                        update: ProjectUpdate::Removed(Some(project.into())),
                    })?;
                }

                Ok(())
            }
        }
    }
}
