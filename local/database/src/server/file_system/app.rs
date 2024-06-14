use crate::{
    event::{self as update, Update},
    server::state,
    Database,
};
use std::assert_matches::assert_matches;
use syre_fs_watcher::{event, EventKind};
use syre_local::TryReducible;

impl Database {
    pub fn handle_fs_event_config(&mut self, event: syre_fs_watcher::Event) -> Vec<crate::Update> {
        let EventKind::Config(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::Config::Created => todo!(),
            event::Config::Removed => todo!(),
            event::Config::Modified(_) => todo!(),
            event::Config::ProjectManifest(kind) => {
                self.handle_fs_event_app_project_manifest(event)
            }
            event::Config::UserManifest(_) => todo!(),
        }
    }
}

impl Database {
    fn handle_fs_event_app_project_manifest(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::Config(event::Config::ProjectManifest(kind)) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::StaticResourceEvent::Created => {
                self.handle_fs_event_app_project_manifest_created(event)
            }
            event::StaticResourceEvent::Removed => {
                self.handle_fs_event_app_project_manifest_removed(event)
            }
            event::StaticResourceEvent::Modified(kind) => match kind {
                event::ModifiedKind::Data => {
                    self.handle_fs_event_app_project_manifest_modified(event)
                }
                event::ModifiedKind::Other => todo!(),
            },
        }
    }

    fn handle_fs_event_app_project_manifest_created(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        use state::config::{action::Manifest as ManifestAction, Action as ConfigAction};

        assert_matches!(
            event.kind(),
            EventKind::Config(event::Config::ProjectManifest(
                event::StaticResourceEvent::Created
            ))
        );

        match syre_local::system::collections::ProjectManifest::load_or_default() {
            Ok(manifest) => {
                self.fs_command_client.clear_projects();

                for path in manifest.iter() {
                    self.fs_command_client.watch(path);
                }

                self.state
                    .try_reduce(
                        ConfigAction::ProjectManifest(ManifestAction::SetOk((*manifest).clone()))
                            .into(),
                    )
                    .unwrap();

                vec![Update::app(
                    update::ProjectManifest::Added((*manifest).clone()),
                    event.id().clone(),
                )]
            }

            Err(err) => {
                todo!();
            }
        }
    }

    fn handle_fs_event_app_project_manifest_removed(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::Config(event::Config::ProjectManifest(
                event::StaticResourceEvent::Removed
            ))
        );

        todo!();
    }

    fn handle_fs_event_app_project_manifest_modified(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::Config(event::Config::ProjectManifest(
                event::StaticResourceEvent::Modified(event::ModifiedKind::Data),
            ))
        );

        todo!();
    }
}
