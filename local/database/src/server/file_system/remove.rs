//! Handle file system events.
use crate::error::Result;
use crate::events::{Asset as AssetUpdate, Graph as GraphUpdate, Update};
use crate::server::Database;
use notify::{self, event::RemoveKind, EventKind};
use notify_debouncer_full::DebouncedEvent;
use thot_core::types::ResourceId;
use thot_local::graph::ContainerTreeTransformer;

impl Database {
    /// Handle [`notify::event::RemoveKind`] events.
    #[tracing::instrument(skip(self))]
    pub fn handle_file_system_event_remove(&mut self, event: DebouncedEvent) -> Result {
        tracing::debug!(?event);
        let EventKind::Remove(kind) = event.event.kind else {
            panic!("invalid event kind");
        };

        let [path] = &event.event.paths[..] else {
            panic!("invalid paths");
        };

        if path.components().any(|seg| seg.as_os_str() == ".thot") {
            todo!();
        }

        // Assume that relative segments are resolved in file paths.
        // On Windows paths are canonicalized to UNC when inserted into database.
        // Can not use `fs::canonicalize` on `from` because file no longer exists,
        // so must canonicalize by hand.
        #[cfg(target_os = "windows")]
        let path = thot_local::common::ensure_windows_unc(path);

        match kind {
            RemoveKind::File => {
                let Some(asset) = self.store.get_path_container(&path).cloned() else {
                    return Ok(());
                };

                self.handle_remove_asset(asset)?;
                Ok(())
            }
            RemoveKind::Folder => {
                let Some(container) = self.store.get_path_container(&path).cloned() else {
                    return Ok(());
                };

                self.handle_remove_container(container)?;
                Ok(())
            }

            RemoveKind::Any => {
                if let Some(container) = self.store.get_path_container(&path).cloned() {
                    self.handle_remove_container(container)?;
                    Ok(())
                } else if let Some(asset) = self.store.get_path_asset_id(&path).cloned() {
                    self.handle_remove_asset(asset)?;
                    Ok(())
                } else {
                    tracing::debug!("`{path:?}` is not a resource");
                    Ok(())
                }
            }

            RemoveKind::Other => {
                tracing::debug!("other {:?}", event);
                todo!();
            }
        }
    }

    /// Removes the subgraph of the Container.
    /// Publishes the `ResourceId` of the removed subgraph's root.
    fn handle_remove_container(&mut self, container: ResourceId) -> Result {
        let project = self
            .store
            .get_container_project(&container)
            .unwrap()
            .clone();

        let graph = self.store.remove_subgraph(&container)?;
        let graph = ContainerTreeTransformer::local_to_core(&graph);
        self.publish_update(&Update::Project {
            project,
            update: GraphUpdate::Removed(graph).into(),
        })?;

        Ok(())
    }

    fn handle_remove_asset(&mut self, asset: ResourceId) -> Result {
        let container = self.store.get_asset_container_id(&asset).unwrap();
        let project = self.store.get_container_project(container).unwrap().clone();
        self.store.remove_asset(&asset)?;
        self.publish_update(&Update::Project {
            project,
            update: AssetUpdate::Removed(asset).into(),
        })?;

        Ok(())
    }
}
