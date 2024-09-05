use crate::{
    common,
    event::{self as update, Update},
    server, state, Database,
};
use std::{assert_matches::assert_matches, io, path::Path};
use syre_fs_watcher::{event, EventKind};
use syre_local::{error::IoSerde, loader, TryReducible};

impl Database {
    pub(super) fn handle_fs_event_folder(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::Folder(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::ResourceEvent::Modified(_) => self.handle_fs_event_folder_modified(event),
            _ => todo!(),
        }
    }
}

impl Database {
    fn handle_fs_event_folder_modified(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::Folder(event::ResourceEvent::Modified(kind)) = event.kind() else {
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
