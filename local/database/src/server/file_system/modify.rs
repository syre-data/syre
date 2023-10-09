//! Handle file system events.
use crate::server::Database;
use crate::update::{Container as ContainerUpdate, Update};
use crate::Result;
use notify::{self, EventKind};
use notify_debouncer_full::DebouncedEvent;
use std::path::{Path, PathBuf};
use thot_core::types::ResourceId;
#[cfg(target_os = "windows")]
use thot_local::constants::WINDOWS_UNC_PREFIX;

impl Database {
    /// Handle [`notify::event::EventKind::ModifyKind`] events.
    #[tracing::instrument(skip(self))]
    pub fn handle_file_system_event_modify(&mut self, event: DebouncedEvent) {
        let EventKind::Modify(kind) = event.event.kind else {
            panic!("invalid event kind");
        };

        match kind {
            notify::event::ModifyKind::Name(rename_mode) => match rename_mode {
                notify::event::RenameMode::Both => {
                    let [from, to] = &event.event.paths[..] else {
                        panic!("invalid paths");
                    };

                    if to.is_file() {
                        self.update_asset_file_path(from, to).unwrap();
                    } else if to.is_dir() {
                        let container = self.update_container_path(from, to).unwrap();
                        self.publish_update(
                            &ContainerUpdate::PathChange {
                                container,
                                path: to.clone(),
                            }
                            .into(),
                        )
                        .unwrap();
                    } else {
                        panic!("unknown path resource");
                    }
                }

                notify::event::RenameMode::From => {
                    tracing::debug!("from {:?}", event);
                }

                notify::event::RenameMode::To => {
                    tracing::debug!("to {:?}", event);
                }

                _ => {
                    tracing::debug!("other {:?}", event)
                }
            },

            _ => {}
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
        tracing::debug!(?from);

        // Assume that relative segments are resolved in file paths.
        // On Windows paths are canonicalized to UNC when inserted.
        // Can not use `fs::canonicalize` on `from` because file no longer exists,
        // so mush canonicalize by hand.
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
