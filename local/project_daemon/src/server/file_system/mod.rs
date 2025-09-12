mod analysis;
mod app;
mod asset;
mod container;
mod file;
mod folder;
mod graph;
mod project;

use crate::{Daemon, Update};
use std::path::Path;
use syre_fs_daemon::{EventKind, event};
use syre_local as local;

impl Daemon {
    pub fn process_file_system_events(
        &mut self,
        events: Vec<syre_fs_daemon::Event>,
    ) -> Vec<Update> {
        events
            .into_iter()
            .flat_map(|event| self.process_event(event))
            .collect()
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
    fn process_event(&mut self, event: syre_fs_daemon::Event) -> Vec<Update> {
        #[cfg(feature = "tracing")]
        tracing::trace!(?event);
        match event.kind() {
            EventKind::Config(_) => self.handle_fs_event_config(event),
            EventKind::Project(_) => self.handle_fs_event_project(event),
            EventKind::Graph(_) => self.handle_fs_event_graph(event),
            EventKind::GraphResource(_) => self.handle_fs_event_graph_resource(event),
            EventKind::Container(_) => self.handle_fs_event_container(event),
            EventKind::AssetFile(_) => self.handle_fs_event_asset_file(event),
            EventKind::AnalysisFile(_) => self.handle_fs_event_analysis_file(event),
            EventKind::File(_) => self.handle_fs_event_file(event),
            EventKind::Folder(_) => self.handle_fs_event_folder(event),
            EventKind::Nonresource(_) => self.handle_fs_event_nonresource(event),
            EventKind::Any(_) => self.handle_fs_event_any(event),
            EventKind::OutOfSync => todo!(),
        }
    }
}

impl Daemon {
    pub(super) fn handle_fs_event_nonresource(
        &mut self,
        event: syre_fs_daemon::Event,
    ) -> Vec<Update> {
        let EventKind::Nonresource(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::Nonresource::Removed => vec![],
        }
    }

    pub(super) fn handle_fs_event_any(&mut self, event: syre_fs_daemon::Event) -> Vec<Update> {
        let EventKind::Any(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::Any::Removed => todo!(),
        }
    }
}

/// # Returns
/// Number of occurances of [`app dir`](syre_local::constants::APP_DIR) in the path.
fn path_app_dir_count(path: impl AsRef<Path>) -> usize {
    path.as_ref()
        .components()
        .filter(|segment| match segment {
            std::path::Component::Normal(segment) => {
                segment.to_str().unwrap() == local::constants::APP_DIR
            }
            _ => false,
        })
        .count()
}
