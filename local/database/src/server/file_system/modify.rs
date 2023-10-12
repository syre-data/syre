//! Handle file system events.
use crate::events::{Container as ContainerUpdate, Project as ProjectUpdate, Update};
use crate::server::Database;
use crate::Result;
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify_debouncer_full::DebouncedEvent;
use std::path::{Path, PathBuf};
use thot_core::types::ResourceId;
#[cfg(target_os = "windows")]
use thot_local::constants::WINDOWS_UNC_PREFIX;

impl Database {
    /// Handle [`notify::event::ModifyKind`] events.
    #[tracing::instrument(skip(self))]
    pub fn handle_file_system_event_modify(&mut self, event: DebouncedEvent) -> Result {
        let EventKind::Modify(kind) = event.event.kind else {
            panic!("invalid event kind");
        };

        match kind {
            ModifyKind::Name(rename_mode) => match rename_mode {
                RenameMode::Both => {
                    let [from, to] = &event.event.paths[..] else {
                        panic!("invalid paths");
                    };

                    if to.is_file() {
                        self.update_asset_file_path(from, to)?;
                    } else if to.is_dir() {
                        let container = self.update_container_path(from, to)?;
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
                                container,
                                properties,
                            }
                            .into(),
                        })?
                    } else {
                        panic!("unknown path resource");
                    }

                    Ok(())
                }

                RenameMode::From => {
                    tracing::debug!("from {:?}", event);
                    todo!();
                }

                RenameMode::To => {
                    tracing::debug!("to {:?}", event);
                    todo!();
                }

                _ => {
                    tracing::debug!("other {:?}", event);
                    todo!();
                }
            },

            _ => Ok(()),
        }
    }

    fn update_asset_file_path(&mut self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result {
        todo!();
    }

    /// Updates a `Container`'s path.
    /// Syncs name with path.
    ///
    /// # Returns
    /// `ResouceId` of the affected `Container`.
    #[tracing::instrument(skip(self, from, to))]
    fn update_container_path(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<ResourceId> {
        let to = to.as_ref();
        let from = PathBuf::from(from.as_ref());

        // Assume that relative segments are resolved in file paths.
        // On Windows paths are canonicalized to UNC when inserted.
        // Can not use `fs::canonicalize` on `from` because file no longer exists,
        // so must canonicalize by hand.
        #[cfg(target_os = "windows")]
        let from = if from.starts_with(WINDOWS_UNC_PREFIX) {
            from
        } else {
            // Must prefix UNC path as `str` because using `Path`s strips it.
            let mut f = WINDOWS_UNC_PREFIX.to_string();
            f.push_str(from.to_str().unwrap());
            PathBuf::from(f)
        };

        let cid = self
            .store
            .get_path_container(from.as_ref())
            .unwrap()
            .clone();

        let container = self.store.get_container_mut(&cid).unwrap();
        container.properties.name = to.file_name().unwrap().to_str().unwrap().to_string();
        self.store.update_subgraph_path(&cid, to)?; // must update graph paths before saving container

        let container = self.store.get_container(&cid).unwrap();
        container.save()?;

        Ok(cid)
    }
}
