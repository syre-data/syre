use crate::{
    common,
    event::{self as update, Update},
    server::{self, state::project::graph},
    state, Database,
};
use std::assert_matches::assert_matches;
use syre_fs_watcher::{event, EventKind};
use syre_local::{self as local, TryReducible};

impl Database {
    pub(super) fn handle_fs_event_graph(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::Graph(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::Graph::Created => self.handle_fs_event_graph_created(event),
            event::Graph::Removed => todo!(),
            event::Graph::Moved => self.handle_fs_event_graph_moved(event),
            event::Graph::Modified(_) => todo!(),
        }
    }

    pub(super) fn handle_fs_event_graph_resource(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::GraphResource(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::GraphResource::Removed => self.handle_fs_event_graph_resource_removed(event),
        }
    }
}

impl Database {
    fn handle_fs_event_graph_created(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        assert_matches!(event.kind(), EventKind::Graph(event::Graph::Created));
        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        assert!(project_state.graph().is_present());
        let state::DataResource::Ok(project_properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let data_root_path = project.path().join(&project_properties.data_root);
        let parent_path =
            common::container_graph_path(&data_root_path, path.parent().unwrap()).unwrap();
        let subgraph = graph::State::load(path).unwrap();
        let subgraph_state = subgraph.as_graph();

        let project_path = project.path().clone();
        let project_id = project_properties.rid().clone();

        #[cfg(target_os = "windows")]
        {
            let state::FolderResource::Present(graph) = project_state.graph() else {
                unreachable!();
            };

            let root = subgraph.root().lock().unwrap();
            let root_path = parent_path.join(root.name());
            drop(root);
            if graph.find(&root_path).unwrap().is_some() {
                tracing::trace!("{root_path:?} already exists");
                self.state
                    .try_reduce(server::state::Action::Project {
                        path: project_path.clone(),
                        action: server::state::project::action::Graph::Remove(root_path).into(),
                    })
                    .unwrap();

                self.state
                    .try_reduce(server::state::Action::Project {
                        path: project_path.clone(),
                        action: server::state::project::action::Graph::Insert {
                            parent: parent_path.clone(),
                            graph: subgraph,
                        }
                        .into(),
                    })
                    .unwrap();
            } else {
                self.state
                    .try_reduce(server::state::Action::Project {
                        path: project_path.clone(),
                        action: server::state::project::action::Graph::Insert {
                            parent: parent_path.clone(),
                            graph: subgraph,
                        }
                        .into(),
                    })
                    .unwrap();
            }
        }

        #[cfg(not(target_os = "windows"))]
        self.state
            .try_reduce(server::state::Action::Project {
                path: project_path.clone(),
                action: server::state::project::action::Graph::Insert {
                    parent: parent_path.clone(),
                    graph: subgraph,
                }
                .into(),
            })
            .unwrap();

        vec![Update::project_with_id(
            project_id,
            project_path,
            update::Graph::Inserted {
                parent: parent_path,
                graph: subgraph_state,
            }
            .into(),
            event.id().clone(),
        )]
    }

    fn handle_fs_event_graph_moved(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::Graph(event::Graph::Moved) = event.kind() else {
            panic!("invalid event kind");
        };

        let [from, to] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let project = self.state.find_resource_project_by_path(from).unwrap();
        let state::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        assert!(project_state.graph().is_present());
        let state::DataResource::Ok(project_properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let data_root_path = project.path().join(&project_properties.data_root);
        let from_path = common::container_graph_path(&data_root_path, from).unwrap();
        let to_path = common::container_graph_path(&data_root_path, to).unwrap();

        let project_path = project.path().clone();
        let project_id = project_properties.rid().clone();
        self.state
            .try_reduce(server::state::Action::Project {
                path: project_path.clone(),
                action: server::state::project::action::Graph::Move {
                    from: from_path.clone(),
                    to: to_path.clone(),
                }
                .into(),
            })
            .unwrap();

        vec![Update::project_with_id(
            project_id.clone(),
            project_path.clone(),
            update::Graph::Moved {
                from: from_path,
                to: to_path,
            }
            .into(),
            event.id().clone(),
        )]
    }
}

impl Database {
    fn handle_fs_event_graph_resource_removed(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::GraphResource(event::GraphResource::Removed)
        );
        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        let state::DataResource::Ok(project_properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let state::FolderResource::Present(graph) = project_state.graph() else {
            panic!("invalid state");
        };

        let data_root_path = project.path().join(&project_properties.data_root);

        let graph_path = common::container_graph_path(&data_root_path, path).unwrap();
        let project_path = project.path().clone();
        let project_id = project_properties.rid().clone();

        if let Some(_container) = graph.find(&graph_path).unwrap() {
            self.state
                .try_reduce(server::state::Action::Project {
                    path: project_path.clone(),
                    action: server::state::project::action::Graph::Remove(graph_path.clone())
                        .into(),
                })
                .unwrap();

            return vec![Update::project_with_id(
                project_id.clone(),
                project_path.clone(),
                update::Graph::Removed(graph_path).into(),
                event.id().clone(),
            )];
        }

        // TODO: When using buckets, must get the nearest container.
        // At the time this is written, buckets are not yet implemented though.
        let parent_container_path = path.parent().unwrap();
        let parent_container_graph_path =
            common::container_graph_path(&data_root_path, parent_container_path).unwrap();
        let rel_path = graph_path
            .strip_prefix(&parent_container_graph_path)
            .unwrap();

        let parent_node = graph.find(&parent_container_graph_path).unwrap().unwrap();
        let parent_state = parent_node.lock().unwrap();
        if let state::DataResource::Ok(assets) = parent_state.assets().clone() {
            if let Some(asset) = assets.iter().find(|asset| asset.path == rel_path) {
                let asset = asset.rid().clone();
                drop(parent_state);

                self.state
                    .try_reduce(server::state::Action::Project {
                        path: project_path.clone(),
                        action: server::state::project::action::Action::Container {
                            path: parent_container_graph_path.clone(),
                            action: server::state::project::action::Container::Asset {
                                rid: asset.clone(),
                                action: server::state::project::action::Asset::SetAbsent,
                            },
                        },
                    })
                    .unwrap();

                if self.config.handle_fs_resource_changes() {
                    tracing::debug!(?parent_container_path);
                    let mut local_assets =
                        local::project::resources::Assets::load_from(parent_container_path)
                            .unwrap();
                    local_assets.retain(|local_asset| *local_asset.rid() != asset);
                    local_assets.save().unwrap();
                    return vec![];
                } else {
                    return vec![Update::project_with_id(
                        project_id.clone(),
                        project_path.clone(),
                        update::Project::Asset {
                            container: parent_container_graph_path,
                            asset: asset.clone(),
                            update: update::Asset::FileRemoved,
                        },
                        event.id().clone(),
                    )];
                }
            }
        }

        // TODO: It could be that a file unassociated with a graph resource was
        // removed.
        // e.g. A file in a container folder that is not registered as an Asset.
        // Need to decide what to do in this case.
        panic!("expected a graph resource");
    }
}
