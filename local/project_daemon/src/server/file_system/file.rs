use crate::{Daemon, event::Update};
use syre_fs_daemon::{EventKind, event};

impl Daemon {
    pub(super) fn handle_fs_event_file(&mut self, event: syre_fs_daemon::Event) -> Vec<Update> {
        let EventKind::File(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::ResourceEvent::Created => self.handle_fs_event_file_created(event),
            event::ResourceEvent::Modified(_) => self.handle_fs_event_file_modified(event),
            event::ResourceEvent::Removed => self.handle_fs_event_file_removed(event),
            event::ResourceEvent::Renamed => self.handle_fs_event_file_renamed(event),
            event::ResourceEvent::Moved => todo!(),
            event::ResourceEvent::MovedProject => todo!(),
        }
    }
}

impl Daemon {
    fn handle_fs_event_file_created(&mut self, event: syre_fs_daemon::Event) -> Vec<Update> {
        std::assert_matches!(event.kind(), EventKind::File(event::ResourceEvent::Created));

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        // TODO: May want to perform additional checks on if file is a resource worth watching.
        vec![]
    }

    fn handle_fs_event_file_renamed(&mut self, event: syre_fs_daemon::Event) -> Vec<Update> {
        std::assert_matches!(event.kind(), EventKind::File(event::ResourceEvent::Renamed));

        let [from, to] = &event.paths()[..] else {
            panic!("invalid paths");
        };
        #[cfg(feature = "tracing")]
        tracing::info!("file renamed from {from:?} to {to:?}");

        // TODO: May want to perform additional checks on if file is a resource worth watching.
        vec![]
    }

    fn handle_fs_event_file_removed(&mut self, event: syre_fs_daemon::Event) -> Vec<Update> {
        std::assert_matches!(event.kind(), EventKind::File(event::ResourceEvent::Removed));

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };
        #[cfg(feature = "tracing")]
        tracing::info!("file removed {path:?}");

        // TODO: May want to perform additional checks on if file is a resource worth watching.
        vec![]
    }

    fn handle_fs_event_file_modified(&mut self, event: syre_fs_daemon::Event) -> Vec<Update> {
        let EventKind::File(event::ResourceEvent::Modified(kind)) = event.kind() else {
            panic!("invalid event kind");
        };

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        match kind {
            event::ModifiedKind::Data => {
                #[cfg(feature = "tracing")]
                tracing::info!("file modified data {path:?}");
                vec![]
            }
            event::ModifiedKind::Other => {
                #[cfg(feature = "tracing")]
                tracing::info!("file modified other {path:?}");
                vec![]
            }
        }
    }
}
