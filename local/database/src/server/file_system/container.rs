//! Handle [`syre::Container`](ContainerEvent) events.
use super::event::app::Container as ContainerEvent;
use crate::event::{Container as ContainerUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::path::PathBuf;
use syre_core::types::ResourceId;

impl Database {
    pub fn handle_app_event_container(&mut self, event: &ContainerEvent) -> Result {
        match event {
            ContainerEvent::Renamed { container, name } => {
                self.update_container_name(&container, name.clone())?;
                let project = self
                    .store
                    .get_container_project(&container)
                    .unwrap()
                    .clone();

                let properties = self
                    .store
                    .get_container(&container)
                    .unwrap()
                    .properties
                    .clone();

                self.publish_update(&Update::Project {
                    project,
                    update: ContainerUpdate::Properties {
                        container: container.clone(),
                        properties,
                    }
                    .into(),
                })?;

                Ok(())
            }
        }
    }

    /// Updates a subgraph's path.
    /// Syncs the root `Container`'s name with path.
    ///
    /// # Returns
    /// `ResouceId` of the affected `Container`.
    #[tracing::instrument(skip(self))]
    fn update_container_name(&mut self, root: &ResourceId, name: PathBuf) -> Result {
        let rid = root.clone();
        let root = self.store.get_container_mut(&root).unwrap();
        root.properties.name = name.to_str().unwrap().to_string();

        let mut path = root.base_path().to_path_buf();
        path.set_file_name(name);
        self.store.update_subgraph_path(&rid, path)?; // must update graph paths before saving container

        let root = self.store.get_container(&rid).unwrap();
        root.save()?;
        Ok(())
    }
}
