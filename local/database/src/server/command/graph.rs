//! Implementation of graph related functionality.
use super::super::Database;
use crate::command::graph::NewChildArgs;
use crate::command::GraphCommand;
use crate::server::store::ContainerTree;
use crate::{Error, Result};
use serde_json::Value as JsValue;
use std::path::Path;
use std::result::Result as StdResult;
use thot_core::error::{Error as CoreError, ProjectError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;
use thot_local::common::unique_file_name;
use thot_local::graph::{ContainerTreeDuplicator, ContainerTreeLoader, ContainerTreeTransformer};
use thot_local::project::container;
use thot_local::project::resources::container::Container;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_graph(&mut self, cmd: GraphCommand) -> JsValue {
        match cmd {
            GraphCommand::Load(project) => {
                let graph = match self.load_project_graph(&project) {
                    Ok(graph) => graph,
                    Err(err) => {
                        let err: Result<ResourceTree<CoreContainer>> = Err(err.into());
                        return serde_json::to_value(err)
                            .expect("could not convert `Result` to JsValue");
                    }
                };

                let graph = ContainerTreeTransformer::local_to_core(graph);
                let res: Result<ResourceTree<CoreContainer>> = Ok(graph);
                serde_json::to_value(res).expect("could not convert graph into JsValue")
            }

            GraphCommand::Get(root) => {
                let Some(graph) = self.store.get_container_graph(&root) else {
                    let res: Result<Option<ResourceTree<CoreContainer>>> = Ok(None);
                    return serde_json::to_value(res).expect("could not convert to JsValue");
                };

                let graph = ContainerTreeTransformer::local_to_core(graph);
                serde_json::to_value(graph).expect("could not convert `Result` to JsValue")
            }

            GraphCommand::Duplicate(root) => {
                // duplicate tree
                let res = self.duplicate_container_tree(&root);
                let Ok(rid) = res else {
                    return serde_json::to_value(res)
                        .expect("could not convert `Result` to JsValue");
                };

                // get duplicated tree
                let Some(graph) = self.store.get_container_graph(&rid) else {
                    let err: Result<ResourceTree<CoreContainer>> = Err(CoreError::ResourceError(
                        ResourceError::does_not_exist("graph not found"),
                    )
                    .into());
                    return serde_json::to_value(err).expect("could not convert error to JsValue");
                };

                let graph = ContainerTreeTransformer::subtree_to_core(graph, &rid)
                    .expect("could not convert graph");

                let graph: Result<ResourceTree<CoreContainer>> = Ok(graph);
                serde_json::to_value(graph).expect("could not convert `Result` to JsValue")
            }

            GraphCommand::Parent(rid) => {
                let Some(graph) = self.store.get_container_graph(&rid) else {
                    let err: Result<Option<ResourceId>> =
                        Err(Error::CoreError(CoreError::ResourceError(
                            ResourceError::does_not_exist("`Container` does not exist"),
                        )));

                    return serde_json::to_value(err).expect("could not convert error to JsValue");
                };

                let parent = graph.parent(&rid).unwrap();
                let Some(parent) = parent else {
                    let res: Result<Option<ResourceId>> = Ok(None);
                    return serde_json::to_value(res).expect("could not convert error to JsValue");
                };

                serde_json::to_value(parent).expect("could not convert `Container` to JsValue")
            }

            GraphCommand::Children(rid) => {
                let Some(graph) = self.store.get_container_graph(&rid) else {
                    let res: Option<indexmap::IndexSet<ResourceId>> = None;
                    return serde_json::to_value(res)
                        .expect("could not convert `Container` to JsValue");
                };

                let children = graph.children(&rid).unwrap();
                serde_json::to_value(children).expect("could not convert `Container` to JsValue")
            }
        }
    }

    /// Loads a `Project`'s [`Container`](LocalContainer) tree from settings.
    fn load_project_graph(&mut self, pid: &ResourceId) -> Result<&ContainerTree> {
        let Some(project) = self.store.get_project(pid) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Project` not loaded",
            ))
            .into());
        };

        let Some(data_root) = project.data_root.as_ref() else {
            return Err(
                CoreError::ProjectError(ProjectError::misconfigured("data root not set")).into(),
            );
        };

        if self.store.get_project_graph(pid).is_none() {
            let path = project.base_path().join(data_root);
            let graph: ContainerTree = ContainerTreeLoader::load(&path)?;
            self.store.insert_project_graph(pid.clone(), graph);
        }

        Ok(self.store.get_project_graph(pid).unwrap())
    }

    /// Duplicates a tree in its parent.
    /// Returns the id of the duplicated tree's root node.
    #[tracing::instrument(skip(self))]
    fn duplicate_container_tree(&mut self, rid: &ResourceId) -> Result<ResourceId> {
        let Some(project) = self.store.get_container_project(rid) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` `Project` not loaded",
            ))
            .into());
        };

        let Some(graph) = self.store.get_project_graph(&project) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Project` graph not loaded",
            ))
            .into());
        };

        let Some(root) = graph.get(rid) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist in graph",
            ))
            .into());
        };

        let Some(parent) = graph.parent(rid)?.cloned() else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not have parent",
            ))
            .into());
        };

        // duplicate tree
        let dup_path = unique_file_name(root.base_path().into())?;
        let mut dup = ContainerTreeDuplicator::duplicate_without_assets_to(&dup_path, graph, rid)?;
        let dup_root = dup.root().clone();

        // update root name
        let root = dup.get_mut(&dup_root).unwrap();
        root.properties.name = root
            .base_path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        root.save()?;

        // insert duplicate
        let res = self.store.insert_subgraph(&parent, dup);
        match res {
            Ok(_) => Ok(dup_root),
            Err(err) => Err(err),
        }
    }

    fn new_child(&mut self, parent: &ResourceId, name: String) -> Result<ResourceId> {
        let Some(parent) = self.store.get_container(&parent) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        // create child
        // TODO Ensure unique and valid path.
        let child_path = unique_file_name(parent.base_path().join(&name))?;
        let cid = container::new(&child_path)?;
        let mut child = Container::load_from(child_path)?;
        child.properties.name = name;
        child.save()?;

        // insert into graph
        let child = ContainerTree::new(child);
        self.store.insert_subgraph(&parent.rid.clone(), child)?;
        Ok(cid)
    }
}
