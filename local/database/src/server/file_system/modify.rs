//! Handle file system events.
use crate::events::{
    Asset as AssetUpdate, Container as ContainerUpdate, Project as ProjectUpdate, Update,
};
use crate::server::Database;
use crate::{common, Result};
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify_debouncer_full::DebouncedEvent;
use std::path::{Path, PathBuf};
use thot_core::types::{ResourceId, ResourcePath};

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

                    self.handle_from_to_event(from, to)?;
                    Ok(())
                }

                RenameMode::From => panic!("can not handle individual `From` event"),
                RenameMode::To => panic!("can not handle individual `To` event"),

                _ => {
                    tracing::debug!("other {:?}", event);
                    todo!();
                }
            },

            _ => Ok(()),
        }
    }

    pub fn handle_from_to_event(&mut self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result {
        let from = from.as_ref();
        let to = to.as_ref();

        if to.is_file() {
            let asset = self.update_asset_file_path(from, to)?;
            let container = self.store.get_asset_container(&asset).unwrap();
            let project = self
                .store
                .get_container_project(&container.rid)
                .unwrap()
                .clone();

            let path = container.assets.get(&asset).unwrap().path.clone();
            self.publish_update(&Update::Project {
                project,
                update: AssetUpdate::PathChanged { asset, path }.into(),
            })?
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

    /// Updates an `Asset`'s file path.
    ///
    /// # Returns
    /// `ResourceId` of the `Asset`.
    fn update_asset_file_path(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<ResourceId> {
        let to = to.as_ref();
        let from = from.as_ref();

        // Assume that relative segments are resolved in file paths.
        // On Windows paths are canonicalized to UNC when inserted.
        // Can not use `fs::canonicalize` on `from` because file no longer exists,
        // so must canonicalize by hand.
        #[cfg(target_os = "windows")]
        let from = common::ensure_windows_unc(from);

        self.store.update_asset_path(from, to)?;
        let aid = self.store.get_path_asset_id_canonical(to).unwrap().clone();
        Ok(aid)
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
        let from = from.as_ref();

        // Assume that relative segments are resolved in file paths.
        // On Windows paths are canonicalized to UNC when inserted.
        // Can not use `fs::canonicalize` on `from` because file no longer exists,
        // so must canonicalize by hand.
        #[cfg(target_os = "windows")]
        let from = common::ensure_windows_unc(from);

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
