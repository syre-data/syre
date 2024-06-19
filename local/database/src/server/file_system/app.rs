use crate::{
    event::{self as update, Update},
    server::state,
    Database,
};
use std::{assert_matches::assert_matches, io, path::PathBuf};
use syre_fs_watcher::{event, EventKind};
use syre_local::{
    project::resources::{Analyses, Project},
    TryReducible,
};

impl Database {
    pub fn handle_fs_event_config(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
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

        match syre_local::system::collections::ProjectManifest::load_from(
            self.config.project_manifest(),
        ) {
            Ok(manifest) => {
                self.fs_command_client.clear_projects();
                for path in manifest.iter() {
                    self.fs_command_client.watch(path).unwrap();
                }

                self.state
                    .try_reduce(
                        ConfigAction::ProjectManifest(ManifestAction::SetOk((*manifest).clone()))
                            .into(),
                    )
                    .unwrap();

                for path in manifest.iter() {
                    self.state
                        .try_reduce(state::Action::InsertProject(
                            state::project::State::load_from(path),
                        ))
                        .unwrap();
                }

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
        use state::config::{action::Manifest as ManifestAction, Action as ConfigAction};

        assert_matches!(
            event.kind(),
            EventKind::Config(event::Config::ProjectManifest(
                event::StaticResourceEvent::Modified(event::ModifiedKind::Data),
            ))
        );
        assert_eq!(event.paths().len(), 1);
        assert_eq!(event.paths()[0], *self.config.project_manifest());

        let manifest = syre_local::system::collections::ProjectManifest::load_from(
            self.config.project_manifest(),
        );

        let state = self.state.app().project_manifest();
        match (manifest, state) {
            (Ok(manifest), Ok(state)) => {
                let mut added = vec![];
                for path in manifest.iter() {
                    if !state.contains(path) {
                        added.push(path.clone());
                    }
                }

                let mut removed = vec![];
                for path in state.iter() {
                    if !manifest.contains(path) {
                        removed.push(path.clone());
                    }
                }

                self.state
                    .try_reduce(
                        ConfigAction::ProjectManifest(ManifestAction::SetOk((*manifest).clone()))
                            .into(),
                    )
                    .unwrap();

                let (added, invalid): (Vec<_>, Vec<_>) =
                    added.into_iter().partition(|path| path.is_absolute());

                for path in added.iter() {
                    let project = state::project::State::load_from(path);
                    self.state
                        .try_reduce(state::Action::InsertProject(project))
                        .unwrap();

                    self.fs_command_client.watch(path.clone()).unwrap();
                }

                for path in removed.iter() {
                    self.state
                        .try_reduce(state::Action::RemoveProject(path.clone()))
                        .unwrap();

                    self.fs_command_client.unwatch(path.clone()).unwrap();
                }

                for path in invalid {
                    todo!();
                }

                let mut updates = vec![];
                if added.len() > 0 {
                    updates.push(Update::app(
                        update::ProjectManifest::Added(added),
                        event.id().clone(),
                    ));
                }

                if removed.len() > 0 {
                    updates.push(Update::app(
                        update::ProjectManifest::Removed(removed),
                        event.id().clone(),
                    ));
                }

                updates
            }

            (Ok(manifest), Err(_state)) => {
                self.state
                    .try_reduce(
                        ConfigAction::ProjectManifest(ManifestAction::SetOk((*manifest).clone()))
                            .into(),
                    )
                    .unwrap();

                let mut added = vec![];
                let mut invalid = vec![];
                for path in manifest.iter() {
                    if path.is_absolute() {
                        if !self
                            .state
                            .projects()
                            .iter()
                            .any(|project| project.path() == path)
                        {
                            self.state
                                .try_reduce(state::Action::InsertProject(
                                    state::project::State::load_from(path),
                                ))
                                .unwrap();

                            added.push(path.clone());
                        }
                    } else {
                        invalid.push(path.clone());
                    }
                }

                let mut removed = vec![];
                let project_paths = self
                    .state
                    .projects()
                    .iter()
                    .map(|project| project.path().clone())
                    .collect::<Vec<_>>();

                for project in project_paths {
                    if !manifest.contains(&project) {
                        self.state
                            .try_reduce(state::Action::RemoveProject(project.clone()))
                            .unwrap();
                    }

                    removed.push(project);
                }

                let mut updates = vec![Update::app(
                    update::ProjectManifest::Repaired,
                    event.id().clone(),
                )];

                if added.len() > 0 {
                    updates.push(Update::app(
                        update::ProjectManifest::Added(added),
                        event.id().clone(),
                    ));
                }

                if removed.len() > 0 {
                    updates.push(Update::app(
                        update::ProjectManifest::Removed(removed),
                        event.id().clone(),
                    ));
                }

                if invalid.len() > 0 {
                    todo!();
                }

                updates
            }

            (Err(manifest), Ok(_state)) => {
                self.state
                    .try_reduce(
                        ConfigAction::ProjectManifest(ManifestAction::SetErr(manifest)).into(),
                    )
                    .unwrap();

                vec![Update::app(
                    update::ProjectManifest::Corrupted,
                    event.id().clone(),
                )]
            }

            (Err(manifest), Err(_state)) => {
                self.state
                    .try_reduce(
                        ConfigAction::ProjectManifest(ManifestAction::SetErr(manifest)).into(),
                    )
                    .unwrap();

                vec![]
            }
        }
    }
}
