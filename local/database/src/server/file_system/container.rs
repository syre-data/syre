use crate::{
    event::{self as update, Update},
    server, state, Database,
};
use std::{assert_matches::assert_matches, io, path::Path};
use syre_fs_watcher::{event, EventKind};
use syre_local::{error::IoSerde, loader, TryReducible};

impl Database {
    pub(super) fn handle_fs_event_container(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::Container(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::Container::Renamed => todo!(),
            event::Container::ConfigDir(_) => self.handle_fs_event_container_config_dir(event),
            event::Container::Properties(_) => self.handle_fs_event_container_properties(event),
            event::Container::Settings(_) => todo!(),
            event::Container::Assets(_) => todo!(),
        }
    }
}

impl Database {
    fn handle_fs_event_container_config_dir(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::Container(event::Container::ConfigDir(kind)) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::StaticResourceEvent::Created => {
                self.handle_fs_event_container_config_dir_created(event)
            }
            event::StaticResourceEvent::Removed => {
                self.handle_fs_event_container_config_dir_removed(event)
            }
            event::StaticResourceEvent::Modified(_) => {
                self.handle_fs_event_container_config_dir_modified(event)
            }
        }
    }

    fn handle_fs_event_container_config_dir_created(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::Container(event::Container::ConfigDir(
                event::StaticResourceEvent::Created
            ))
        );

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let base_path = path.parent().unwrap();
        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        let state::FolderResource::Present(graph) = project_state.graph() else {
            panic!("invalid state");
        };

        let state::DataResource::Ok(project_properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let container_graph_path = base_path
            .strip_prefix(project.path().join(&project_properties.data_root))
            .unwrap();
        let container_graph_path = Path::new("/").join(container_graph_path);
        let container_state = graph.find(&container_graph_path).unwrap();
        let container_state = container_state.lock().unwrap();
        assert_matches!(
            container_state.properties(),
            state::DataResource::Err(IoSerde::Io(io::ErrorKind::NotFound))
        );
        assert_matches!(
            container_state.settings(),
            state::DataResource::Err(IoSerde::Io(io::ErrorKind::NotFound))
        );
        assert_matches!(
            container_state.assets(),
            state::DataResource::Err(IoSerde::Io(io::ErrorKind::NotFound))
        );
        drop(container_state);

        let loader::container::State {
            properties,
            settings,
            assets,
        } = loader::container::Loader::load_resources(base_path);

        let mut updates = vec![];
        let project_path = project.path().clone();
        let project_id = project_properties.rid().clone();
        if !matches!(properties, Err(IoSerde::Io(io::ErrorKind::NotFound))) {
            self.state
                .try_reduce(server::state::Action::Project {
                    path: project_path.clone(),
                    action: server::state::project::Action::Container {
                        path: container_graph_path.clone(),
                        action: server::state::project::action::Container::SetProperties(
                            properties.clone(),
                        ),
                    },
                })
                .unwrap();

            updates.push(Update::project_with_id(
                project_id.clone(),
                path,
                update::Project::Container {
                    path: container_graph_path.clone(),
                    update: update::Container::Properties(update::DataResource::Created(
                        properties,
                    )),
                },
                event.id().clone(),
            ));
        }

        if !matches!(settings, Err(IoSerde::Io(io::ErrorKind::NotFound))) {
            self.state
                .try_reduce(server::state::Action::Project {
                    path: project_path.clone(),
                    action: server::state::project::Action::Container {
                        path: container_graph_path.clone(),
                        action: server::state::project::action::Container::SetSettings(
                            settings.clone(),
                        ),
                    },
                })
                .unwrap();

            updates.push(Update::project_with_id(
                project_id.clone(),
                path,
                update::Project::Container {
                    path: container_graph_path.clone(),
                    update: update::Container::Settings(update::DataResource::Created(settings)),
                },
                event.id().clone(),
            ));
        }

        match assets {
            Ok(manifest) => {
                todo!()
            }
            Err(IoSerde::Io(io::ErrorKind::NotFound)) => {}
            Err(err) => {
                self.state
                    .try_reduce(server::state::Action::Project {
                        path: project_path.clone(),
                        action: server::state::project::Action::Container {
                            path: container_graph_path.clone(),
                            action: server::state::project::action::Container::SetAssets(
                                state::DataResource::Err(err.clone()),
                            ),
                        },
                    })
                    .unwrap();

                updates.push(Update::project_with_id(
                    project_id.clone(),
                    path,
                    update::Project::Container {
                        path: container_graph_path.clone(),
                        update: update::Container::Assets(update::DataResource::Created(
                            state::DataResource::Err(err),
                        )),
                    },
                    event.id().clone(),
                ));
            }
        }

        updates
    }

    fn handle_fs_event_container_config_dir_removed(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::Container(event::Container::ConfigDir(
                event::StaticResourceEvent::Removed
            ))
        );

        todo!();
    }
    fn handle_fs_event_container_config_dir_modified(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::Container(event::Container::ConfigDir(
            event::StaticResourceEvent::Modified(kind),
        )) = event.kind()
        else {
            panic!("invalid event kind");
        };

        todo!();
    }
}

impl Database {
    fn handle_fs_event_container_properties(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::Container(event::Container::Properties(kind)) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::StaticResourceEvent::Created => {
                self.handle_fs_event_container_properties_created(event)
            }
            event::StaticResourceEvent::Removed => todo!(),
            event::StaticResourceEvent::Modified(_) => todo!(),
        }
    }

    fn handle_fs_event_container_properties_created(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::Container(event::Container::Properties(
                event::StaticResourceEvent::Created
            ))
        );

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let base_path = path.parent().unwrap().parent().unwrap();
        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        let state::FolderResource::Present(graph) = project_state.graph() else {
            panic!("invalid state");
        };

        let state::DataResource::Ok(project_properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let container_graph_path = base_path
            .strip_prefix(project.path().join(&project_properties.data_root))
            .unwrap();
        let container_graph_path = Path::new("/").join(container_graph_path);
        let container_state = graph.find(&container_graph_path).unwrap();
        let container_state = container_state.lock().unwrap();
        assert_matches!(
            container_state.properties(),
            state::DataResource::Err(IoSerde::Io(io::ErrorKind::NotFound))
        );
        drop(container_state);

        let properties = loader::container::Loader::load_from_only_properties(base_path);
        let project_path = project.path().clone();
        let project_id = project_properties.rid().clone();
        if matches!(properties, Err(IoSerde::Io(io::ErrorKind::NotFound))) {
            vec![]
        } else {
            self.state
                .try_reduce(server::state::Action::Project {
                    path: project_path.clone(),
                    action: server::state::project::Action::Container {
                        path: container_graph_path.clone(),
                        action: server::state::project::action::Container::SetProperties(
                            properties.clone(),
                        ),
                    },
                })
                .unwrap();

            vec![Update::project_with_id(
                project_id.clone(),
                project_path,
                update::Project::Container {
                    path: container_graph_path.clone(),
                    update: update::Container::Properties(update::DataResource::Created(
                        properties,
                    )),
                },
                event.id().clone(),
            )]
        }
    }
}
