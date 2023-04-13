//! Implementation of graph related functionality.
use super::super::Database;
use crate::command::graph::NewChildArgs;
use crate::command::GraphCommand;
use crate::server::store::ContainerTree;
use crate::Error;
use crate::Result;
use serde_json::Value as JsValue;
use thot_core::error::{Error as CoreError, ProjectError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;
use thot_local::common::unique_file_name;
use thot_local::graph::{ContainerTreeDuplicator, ContainerTreeLoader, ContainerTreeTransformer};
use thot_local::project::container;
use thot_local::project::resources::container::{Container, Loader as ContainerLoader};

impl Database {
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
                    return serde_json::to_value(res)
                        .expect("could not convert to JsValue");
                };

                let graph = ContainerTreeTransformer::local_to_core(graph);
                serde_json::to_value(graph).expect("could not convert `Result` to JsValue")
            }

            GraphCommand::Remove(root) => {
                let res = self.store.remove_subgraph(&root);
                serde_json::to_value(res).expect("could not convert `Result` to JsValue")
            }

            GraphCommand::NewChild(NewChildArgs { name, parent }) => {
                let res = self.new_child(&parent, name);
                let Ok(cid) = res else {
                    return serde_json::to_value(res).expect("could not convert error to JsValue");
                };

                let Some(child) = self.store.get_container(&cid) else {
                    let err: Error =  CoreError::ResourceError(ResourceError::DoesNotExist("child `Container` not inserted into graph")).into();
                    return serde_json::to_value(err).expect("could not convert error to JsValue");

                };

                let child: Result<CoreContainer> = Ok((*child).clone());
                serde_json::to_value(child).expect("could not convert child `Container` to JsValue")
            }

            GraphCommand::Duplicate(root) => {
                // duplicate tree
                let res = self.duplicate_container_tree(&root);
                let Ok(rid) = res else {
                    return serde_json::to_value(res).expect("could not convert `Result` to JsValue");
                };

                // get duplicated tree
                let Some(graph) = self.store.get_container_graph(&rid) else {
                    let err: Result<ResourceTree<CoreContainer>> = Err(CoreError::ResourceError(ResourceError::DoesNotExist("graph not found")).into());
                    return serde_json::to_value(err).expect("could not convert error to JsValue");
                };

                let graph = ContainerTreeTransformer::subtree_to_core(graph, &rid)
                    .expect("could not convert graph");

                let graph: Result<ResourceTree<CoreContainer>> = Ok(graph);
                serde_json::to_value(graph).expect("could not convert `Result` to JsValue")
            }
        }
    }

    /// Loads a `Projcet`'s [`Container`](LocalContainer) tree from settings.
    fn load_project_graph(&mut self, pid: &ResourceId) -> Result<&ContainerTree> {
        let Some(project) = self.store.get_project(pid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` not loaded")).into());
        };

        let Some(data_root) = project.data_root.as_ref() else {
            return Err(CoreError::ProjectError(ProjectError::Misconfigured("data root not set")).into());
        };

        if self.store.get_project_graph(pid).is_none() {
            let path = project.base_path().join(data_root);
            let graph: ContainerTree = ContainerTreeLoader::load(&path)?;
            self.store.insert_project_graph(pid.clone(), graph);
        }

        let Some(graph) = self.store.get_project_graph(pid) else {
            return Err(Error::LocalError("could not load `Project` graph".to_string()));
        };

        Ok(graph)
    }

    /// Duplicates a tree in its parent.
    /// Returns the id of the duplicated tree's root node.
    #[tracing::instrument(skip(self))]
    fn duplicate_container_tree(&mut self, rid: &ResourceId) -> Result<ResourceId> {
        let Some(project) = self.store.get_container_project(rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` `Project` not loaded")).into());
        };

        let Some(graph) = self.store.get_project_graph(&project) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` graph not loaded")).into());
        };

        let Some(root) = graph.get(rid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist in graph")).into());
        };

        let Some(parent) = graph.parent(rid)?.cloned() else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not have parent")).into());
        };

        // duplicate tree
        let dup_path = unique_file_name(root.base_path().into())?;
        let dup = ContainerTreeDuplicator::duplicate_to(&dup_path, graph, rid)?;
        let dup_root = dup.root().clone();

        // insert duplicate
        let res = self.store.insert_subgraph(&parent, dup);
        match res {
            Ok(_) => Ok(dup_root),
            Err(err) => Err(err),
        }
    }

    fn new_child(&mut self, parent: &ResourceId, name: String) -> Result<ResourceId> {
        let Some(parent) = self.store.get_container(&parent) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist")).into());
        };

        // create child
        // @todo: Ensure unique and valid path.
        let child_path = unique_file_name(parent.base_path().join(&name))?;
        let cid = container::new(&child_path)?;
        let mut child: Container = ContainerLoader::load_or_create(child_path.into())?.into();
        child.properties.name = Some(name);
        child.save()?;

        // insert into graph
        let child = ContainerTree::new(child);
        self.store.insert_subgraph(&parent.rid.clone(), child)?;
        Ok(cid)
    }
}

#[cfg(test)]
#[path = "./graph_test.rs"]
mod graph_test;
