use crate::{
    common,
    event::{self as update, Update},
    server::{self, state::project::graph},
    state, Database,
};
use std::{assert_matches::assert_matches, io, path::Path};
use syre_fs_watcher::{event, EventKind};
use syre_local::{error::IoSerde, loader, TryReducible};

impl Database {
    pub(super) fn handle_fs_event_graph(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::Graph(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            event::Graph::Created => self.handle_fs_event_graph_created(event),
            event::Graph::Removed => todo!(),
            event::Graph::Moved => todo!(),
            event::Graph::Modified(_) => todo!(),
        }
    }
}

impl Database {
    fn handle_fs_event_graph_created(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        let EventKind::Graph(event::Graph::Created) = event.kind() else {
            panic!("invalid event kind");
        };

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
}
