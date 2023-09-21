//! Handle file system events.
use super::super::Database;
use notify::{self, EventKind};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent};

impl Database {
    /// Handle file system events.
    /// To be used with [`notify::Watcher`]s.
    #[tracing::instrument]
    pub fn handle_file_system_events(event: DebounceEventResult) {
        let events = match event {
            Ok(events) => events,
            Err(errs) => {
                tracing::debug!("watch error: {errs:?}");
                return;
            }
        };

        for event in events.into_iter() {
            tracing::debug!(?event);
            match event.event.kind {
                notify::EventKind::Modify(_) => Database::handle_file_system_event_modify(event),
                _ => {}
            }
        }
    }

    /// Handle [`notify::event::EventKind::ModifyKind`] events.
    #[tracing::instrument]
    fn handle_file_system_event_modify(event: DebouncedEvent) {
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
                    } else if to.is_dir() {
                    } else {
                        panic!("unknown path type end point");
                    }
                }

                notify::event::RenameMode::From => {
                    tracing::debug!("from {:?}", event);
                }

                notify::event::RenameMode::To => {
                    tracing::debug!("from {:?}", event);
                }

                _ => {
                    tracing::debug!("other {:?}", event)
                }
            },

            _ => {}
        }
    }
}
