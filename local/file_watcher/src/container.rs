//! Handle [`syre::Container`](ContainerEvent) events.
use super::event::app::Container as ContainerEvent;
use crate::error::server::UpdateContainer as UpdateContainerError;
use crate::event::{Container as ContainerUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::path::PathBuf;
use std::result::Result as StdResult;
use syre_core::types::ResourceId;
use uuid::Uuid;

impl Database {
    pub fn handle_app_event_container(
        &mut self,
        event: &ContainerEvent,
        event_id: &Uuid,
    ) -> StdResult<Vec<Update>, UpdateContainerError> {
        match event {
            ContainerEvent::Renamed { container, name } => {
                self.update_container_name(&container, name.clone())?;
                let project = self
                    .object_store
                    .get_container_project(&container)
                    .unwrap()
                    .clone();

                let properties = self
                    .object_store
                    .get_container(&container)
                    .unwrap()
                    .properties
                    .clone();

                Ok(vec![Update::project(
                    project,
                    ContainerUpdate::Properties {
                        container: container.clone(),
                        properties,
                    }
                    .into(),
                    event_id.clone(),
                )])
            }
        }
    }

    /// Updates a subgraph's path.
    /// Syncs the root `Container`'s name with path.
    ///
    /// # Returns
    /// `ResouceId` of the affected `Container`.
    #[tracing::instrument(skip(self))]
    fn update_container_name(
        &mut self,
        root: &ResourceId,
        name: PathBuf,
    ) -> StdResult<(), UpdateContainerError> {
        let rid = root.clone();
        let Some(root) = self.object_store.get_container_mut(&root) else {
            return Err(UpdateContainerError::ResourceNotFound);
        };

        root.properties.name = name.to_str().unwrap().to_string();
        let mut path = root.base_path().to_path_buf();
        path.set_file_name(name);

        // must update graph paths before saving container
        if let Err(err) = self.object_store.update_subgraph_path(&rid, path) {
            return Err(UpdateContainerError::Rename(err.into()));
        }

        let root = self.object_store.get_container(&rid).unwrap();
        if let Err(err) = root.save() {
            return Err(UpdateContainerError::Save(err));
        }

        Ok(())
    }
}
