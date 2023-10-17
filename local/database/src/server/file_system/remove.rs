//! Handle file system events.
use crate::error::Result;
use crate::events::{Container as ContainerUpdate, Project as ProjectUpdate, Update};
use crate::server::store::ContainerTree;
use crate::server::Database;
use notify::{self, event::RemoveKind, EventKind};
use notify_debouncer_full::DebouncedEvent;
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::types::ResourceId;
#[cfg(target_os = "windows")]
use thot_local::constants::WINDOWS_UNC_PREFIX;
use thot_local::project::resources::Container;

impl Database {
    /// Handle [`notify::event::RemoveKind`] events.
    #[tracing::instrument(skip(self))]
    pub fn handle_file_system_event_remove(&mut self, event: DebouncedEvent) -> Result {
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
        // On Windows paths are canonicalized to UNC when inserted.
        // Can not use `fs::canonicalize` on `from` because file no longer exists,
        // so must canonicalize by hand.
        #[cfg(target_os = "windows")]
        let path = if path.starts_with(WINDOWS_UNC_PREFIX) {
            path.clone()
        } else {
            // Must prefix UNC path as `str` because using `Path`s strips it.
            let mut p = WINDOWS_UNC_PREFIX.to_string();
            p.push_str(path.to_str().unwrap());
            PathBuf::from(p)
        };

        match kind {
            RemoveKind::File => self.handle_remove_file(path),
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
                } else if let Some(asset) = self.store.get_path_asset_id(&path) {
                    todo!();
                } else {
                    Ok(())
                }
            }

            RemoveKind::Other => {
                tracing::debug!("other {:?}", event);
                todo!();
            }
        }
    }

    fn handle_remove_container(&mut self, container: ResourceId) -> Result {
        let project = self
            .store
            .get_container_project(&container)
            .unwrap()
            .clone();

        self.store.remove_subgraph(&container)?;
        self.publish_update(&Update::Project {
            project,
            update: ProjectUpdate::Container(ContainerUpdate::Removed(container)),
        })?;

        Ok(())
    }

    fn handle_remove_file(&mut self, path: PathBuf) -> Result {
        let asset = self.file_system_remove_asset(path)?;
        todo!();
    }

    fn file_system_remove_asset(&mut self, path: PathBuf) -> Result {
        todo!();
    }
}
