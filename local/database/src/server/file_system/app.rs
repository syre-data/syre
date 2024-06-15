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

                for path in manifest.iter() {
                    self.state
                        .try_reduce(state::Action::InsertProject(
                            self.init_project_state_from_path(path),
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

        let manifest = syre_local::system::collections::ProjectManifest::load_or_default();
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

                if manifest.len() > 0 {
                    vec![Update::app(
                        update::ProjectManifest::Added(manifest.to_vec()),
                        event.id().clone(),
                    )]
                } else {
                    vec![]
                }
            }

            (Err(manifest), Ok(state)) => {
                let state = (*state).clone();
                self.state
                    .try_reduce(
                        ConfigAction::ProjectManifest(ManifestAction::SetErr(manifest)).into(),
                    )
                    .unwrap();

                if state.len() > 0 {
                    vec![Update::app(
                        update::ProjectManifest::Removed(state),
                        event.id().clone(),
                    )]
                } else {
                    vec![]
                }
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

impl Database {
    fn init_project_state_from_path(&self, path: impl Into<PathBuf>) -> state::project::State {
        use state::project;
        use syre_local::{common, project::resources::project::LoadError};

        let path: PathBuf = path.into();
        if !path.is_dir() {
            return project::State::new(path);
        }

        let config = common::app_dir_of(&path);
        if !config.is_dir() {
            return project::State::with_project(
                path,
                project::project::Builder::default().build(),
            );
        }

        let mut state = project::project::Builder::default();
        match Project::load_from(path.clone()) {
            Ok(project) => {
                let (properties, settings, _path) = project.into_parts();
                state.set_properties(properties);
                state.set_settings(settings);
            }

            Err(LoadError {
                properties,
                settings,
            }) => {
                match properties {
                    Ok(properties) => {
                        state.set_properties(properties);
                    }
                    Err(err) => {
                        state.set_properties_err(err);
                    }
                }

                match settings {
                    Ok(settings) => {
                        state.set_settings(settings);
                    }
                    Err(err) => {
                        state.set_settings_err(err);
                    }
                }
            }
        }

        match Analyses::load_from(path.clone()) {
            Ok(analyses) => state.set_analyses(analyses.to_vec()),
            Err(err) => state.set_analyses_err(err),
        };

        return project::State::with_project(path, state.build());
    }
}
