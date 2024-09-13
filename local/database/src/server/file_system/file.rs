use crate::{
    common,
    event::{self as update, Update},
    server, state, Database,
};
use std::{assert_matches::assert_matches, io, path::Path};
use syre_fs_watcher::{event, EventKind};
use syre_local::{error::IoSerde, loader, TryReducible};

impl Database {
    pub(super) fn handle_fs_event_file(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::File(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::ResourceEvent::Created => self.handle_fs_event_file_created(event),
            event::ResourceEvent::Modified(_) => self.handle_fs_event_file_modified(event),
            _ => todo!(),
        }
    }
}

impl Database {
    fn handle_fs_event_file_created(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        assert_matches!(event.kind(), EventKind::File(event::ResourceEvent::Created));

        let [_path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        // TODO: May want to perform additional checks on if file is a resource worth watching.
        vec![]
    }

    fn handle_fs_event_file_modified(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::File(event::ResourceEvent::Modified(kind)) = event.kind() else {
            panic!("invalid event kind");
        };

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        match kind {
            event::ModifiedKind::Data => todo!(),
            event::ModifiedKind::Other => vec![],
        }
    }
}
