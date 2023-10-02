//! Handle file system events.
use crate::server::Database;
use crate::update::{Container as ContainerUpdate, Update};
use crate::Result;
use notify::{self, EventKind};
use notify_debouncer_full::DebouncedEvent;
use std::path::Path;
use thot_core::types::ResourceId;

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
                        let container = self.update_container_name(from, to).unwrap();
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

    fn update_container_name(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<ResourceId> {
        let from = from.as_ref();
        let to = to.as_ref();
        let cid = self.store.get_path_container(from).unwrap().clone();
        let container = self.store.get_container_mut(&cid).unwrap();
        container.properties.name = to.file_name().unwrap().to_str().unwrap().to_string();
        container.save()?;
        self.store.update_subgraph_path(&cid, to)?;

        Ok(cid)
    }
}
