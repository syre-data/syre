//! Handle [`syre::Graph`](GraphEvent) events.
use super::event::app::Graph as GraphEvent;
use crate::error::server::{Rename as RenameError, UpdateContainer as UpdateContainerError};
use crate::event::{Graph as GraphUpdate, Update};
use crate::server::Database;
use crate::Result;
use std::io;
use std::path::PathBuf;
use std::result::Result as StdResult;
use syre_core::types::ResourceId;
use syre_local::graph::{ContainerTreeDuplicator, ContainerTreeTransformer};
use uuid::Uuid;

impl Database {
    pub fn handle_app_event_graph(
        &mut self,
        event: &GraphEvent,
        event_id: &Uuid,
    ) -> Result<Vec<Update>> {
        match event {
            GraphEvent::Moved { root, path } => {
                let project = self
                    .object_store
                    .get_container_project(&root)
                    .unwrap()
                    .clone();

                let parent = self
                    .object_store
                    .get_path_container(&path.parent().unwrap())
                    .unwrap()
                    .clone();

                self.update_subgraph_path(&root, path.clone())?;
                self.object_store.move_subgraph(&root, &parent)?;

                let name = self
                    .object_store
                    .get_container(&root)
                    .unwrap()
                    .properties
                    .name
                    .clone();

                Ok(vec![Update::project(
                    project,
                    GraphUpdate::Moved {
                        root: root.clone(),
                        parent,
                        name,
                    }
                    .into(),
                    event_id.clone(),
                )])
            }

            GraphEvent::Inserted(graph) => {
                // reassign rids
                let mut graph = ContainerTreeDuplicator::duplicate(&graph, graph.root())?;
                let root = graph.root().clone();
                let path = graph.get(&root).unwrap().base_path().to_owned();
                let parent = self
                    .object_store
                    .get_path_container_canonical(path.parent().unwrap())
                    .unwrap()
                    .cloned()
                    .unwrap();

                // sync root container name
                let container = graph.get_mut(&root).unwrap();
                container.properties.name = path.file_name().unwrap().to_str().unwrap().to_string();
                container.save()?;

                // remove scripts if not in project
                let project = self
                    .object_store
                    .get_container_project(&parent)
                    .unwrap()
                    .clone();

                let analyses = self.object_store.get_project_scripts(&project).unwrap();
                for (_, container) in graph.iter_nodes_mut() {
                    container
                        .analyses
                        .retain(|script, _| analyses.contains_key(script));

                    container.save()?;
                }

                // insert graph
                self.object_store.insert_subgraph(&parent, graph)?;
                let project = self
                    .object_store
                    .get_container_project(&root)
                    .unwrap()
                    .clone();

                let graph = self.object_store.get_graph_of_container(&root).unwrap();
                let graph = ContainerTreeTransformer::local_to_core(graph);

                if let Err(err) = self
                    .data_store
                    .graph()
                    .create_subgraph(graph.clone(), parent.clone())
                {
                    tracing::error!(?err);
                }

                Ok(vec![Update::project(
                    project,
                    GraphUpdate::Created { parent, graph }.into(),
                    event_id.clone(),
                )])
            }

            GraphEvent::Copied(graph) => {
                let mut graph = ContainerTreeDuplicator::duplicate(&graph, graph.root())?;
                let root = graph.root().clone();
                let root_container = graph.get_mut(&root).unwrap();
                root_container.properties.name = root_container
                    .base_path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                for container in graph.nodes().values() {
                    container.save()?;
                }

                let path = graph.get(&root).unwrap().base_path();
                let parent = self
                    .object_store
                    .get_path_container_canonical(path.parent().unwrap())
                    .unwrap()
                    .cloned()
                    .unwrap();

                self.object_store.insert_subgraph(&parent, graph)?;

                let project = self
                    .object_store
                    .get_container_project(&root)
                    .unwrap()
                    .clone();

                let graph = self.object_store.get_graph_of_container(&root).unwrap();
                let graph = ContainerTreeTransformer::local_to_core(graph);

                if let Err(err) = self
                    .data_store
                    .graph()
                    .create_subgraph(graph.clone(), parent.clone())
                {
                    tracing::error!(?err);
                }

                Ok(vec![Update::project(
                    project,
                    GraphUpdate::Created { parent, graph }.into(),
                    event_id.clone(),
                )])
            }

            GraphEvent::Removed(root) => {
                let project = self
                    .object_store
                    .get_container_project(&root)
                    .unwrap()
                    .clone();

                let graph = self.object_store.remove_subgraph(&root)?;
                let graph = ContainerTreeTransformer::local_to_core(&graph);

                if let Err(err) = self.data_store.graph().remove(graph.root().clone()) {
                    tracing::error!(?err);
                }

                Ok(vec![Update::project(
                    project,
                    GraphUpdate::Removed(graph).into(),
                    event_id.clone(),
                )])
            }
        }
    }

    /// Updates a subgraph's path.
    /// Syncs the root `Container`'s name with path.
    ///
    /// # Returns
    /// `ResouceId` of the affected `Container`.
    #[tracing::instrument(skip(self))]
    fn update_subgraph_path(
        &mut self,
        root: &ResourceId,
        path: PathBuf,
    ) -> StdResult<(), UpdateContainerError> {
        let rid = root.clone();
        let Some(root) = self.object_store.get_container_mut(&root) else {
            return Err(RenameError::ResourceNotFound.into());
        };

        let Some(name) = path.file_name() else {
            return Err(RenameError::Rename(io::ErrorKind::InvalidFilename).into());
        };

        root.properties.name = name.to_str().unwrap().to_string();
        self.object_store.update_subgraph_path(&rid, path)?; // must update graph paths before saving container

        let root = self.object_store.get_container(&rid).unwrap();
        if let Err(err) = root.save() {
            return Err(UpdateContainerError::Save(err));
        }

        Ok(())
    }
}
