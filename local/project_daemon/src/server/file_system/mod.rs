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
            event::Any::Removed => self.handle_fs_event_any_remove(event),
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

mod any {
    use super::Daemon;
    use crate::{Update, event as update, server, state};
    use syre_fs_daemon::{EventKind, event};
    use syre_local::TryReducible;

    impl Daemon {
        pub(super) fn handle_fs_event_any_remove(
            &mut self,
            event: syre_fs_daemon::Event,
        ) -> Vec<Update> {
            let EventKind::Any(event::Any::Removed) = event.kind() else {
                panic!("invalid event kind");
            };

            let [path] = &event.paths()[..] else {
                panic!("invalid paths");
            };

            if let Some(project) = self
                .state
                .projects()
                .iter()
                .find(|project| project.path() == path)
            {
                let state::FolderResource::Present(project_state) = project.fs_resource() else {
                    panic!("invalid state")
                };
                let project_id = project_state
                    .properties()
                    .map(|properties| properties.rid().clone())
                    .ok();
                self.state
                    .try_reduce(server::state::Action::Project {
                        path: project.path().clone(),
                        action: server::state::project::Action::RemoveFolder,
                    })
                    .unwrap();
                return vec![Update::project(
                    project_id,
                    path.clone(),
                    update::Project::FolderRemoved,
                    event.id().clone(),
                )];
            }

            #[cfg(feature = "tracing")]
            tracing::warn!("unhandled case");
            vec![]
        }
    }
}
