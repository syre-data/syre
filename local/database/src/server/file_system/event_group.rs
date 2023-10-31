//! Try to handle groups of events.
use crate::events::{Asset as AssetUpdate, Update};
use crate::server::Database;
use crate::Result;
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify_debouncer_full::DebouncedEvent;
use thot_core::types::ResourceId;

impl Database {
    /// Filters out uninteresting events.
    pub fn file_system_filter_events(events: Vec<DebouncedEvent>) -> Vec<DebouncedEvent> {
        events
            .into_iter()
            .filter(|event| match event.kind {
                EventKind::Create(_)
                | EventKind::Remove(_)
                | EventKind::Modify(ModifyKind::Name(_)) => true,
                _ => false,
            })
            .collect()
    }

    /// Try to handle events as a group.
    ///
    /// # Returns
    /// `Some` with the `Result` if the events were handled as a group.
    /// `None` if the events were not handled as a group.
    pub fn file_system_try_handle_events_group(
        &mut self,
        events: &Vec<DebouncedEvent>,
    ) -> Option<Result> {
        if let Some((from, to)) = self.file_system_check_rename_from_to(events) {
            return Some(
                self.file_system_handle_rename_from_to_event(&from.paths[0], &to.paths[0]),
            );
        }

        if let Some((asset, path)) = self.file_system_check_asset_move(events) {
            return Some(self.file_system_handle_asset_moved(asset, path));
        }

        None
    }

    fn file_system_check_rename_from_to<'a>(
        &self,
        events: &'a Vec<DebouncedEvent>,
    ) -> Option<(&'a DebouncedEvent, &'a DebouncedEvent)> {
        if events.len() != 2 {
            return None;
        }

        let kinds = events.iter().map(|event| &event.kind).collect::<Vec<_>>();
        let (from, to) = match kinds[..] {
            [EventKind::Modify(ModifyKind::Name(RenameMode::From)), EventKind::Modify(ModifyKind::Name(RenameMode::To))] => {
                (&events[0], &events[1])
            }
            [EventKind::Modify(ModifyKind::Name(RenameMode::To)), EventKind::Modify(ModifyKind::Name(RenameMode::From))] => {
                (&events[1], &events[0])
            }
            _ => return None,
        };

        Some((from, to))
    }

    /// Check if the events represent an `Asset` was moved.
    ///
    /// # Returns
    /// `Some(<asset>, <container>)` if an `Asset` was moved,
    /// `None` otherwise.
    fn file_system_check_asset_move(
        &self,
        events: &Vec<DebouncedEvent>,
    ) -> Option<(ResourceId, ResourceId)> {
        if events.len() != 2 {
            return None;
        }

        let kinds = events.iter().map(|event| &event.kind).collect::<Vec<_>>();
        let (remove, create) = match kinds[..] {
            [EventKind::Remove(_), EventKind::Create(_)] => (&events[0], &events[1]),
            [EventKind::Create(_), EventKind::Remove(_)] => (&events[1], &events[0]),
            _ => return None,
        };

        let Some(asset) = self.store.get_path_asset_id(&remove.paths[0]) else {
            return None;
        };

        let Ok(container) =
            thot_local::project::asset::container_from_path_ancestor(&create.paths[0])
        else {
            return None;
        };

        let Some(container) = self.store.get_path_container(&container) else {
            return None;
        };

        Some((asset.clone(), container.clone()))
    }

    /// Move an `Asset` to a new `Container`.
    fn file_system_handle_asset_moved(
        &mut self,
        asset: ResourceId,
        container: ResourceId,
    ) -> Result {
        let aid = asset.clone();
        let cid = container.clone();
        let project = self
            .store
            .get_container_project(&container)
            .cloned()
            .unwrap();

        let (asset, _asset_path) = self.store.remove_asset(&asset)?.unwrap();
        self.store.add_asset(asset, container)?;

        self.publish_update(&Update::Project {
            project,
            update: AssetUpdate::Moved {
                asset: aid,
                container: cid,
            }
            .into(),
        })?;

        Ok(())
    }
}
