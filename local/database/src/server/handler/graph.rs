//! Implementation of graph related functionality.
use super::super::Database;
use crate::command::graph::NewChildArgs;
use crate::command::GraphCommand;
use crate::server::store::ContainerTree;
use crate::Error;
use crate::Result;
use serde_json::Value as JsValue;
use settings_manager::LocalSettings;
use thot_core::error::{Error as CoreError, ProjectError, ResourceError};
use thot_core::graph::ResourceTree as CoreResourceTree;
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;
use thot_local::common::unique_file_name;
use thot_local::project::container;
use thot_local::project::resources::Container;

type CoreContainerTree = CoreResourceTree<CoreContainer>;

impl Database {
    pub fn handle_command_graph(&mut self, cmd: GraphCommand) -> JsValue {
        match cmd {
            GraphCommand::Load(project) => {
                let res = self.load_project_graph(&project);
                if let Err(err) = res {
                    let err: Result = Err(err.into());
                    return serde_json::to_value(err)
                        .expect("could not convert `Result` to JsValue");
                };

                let graph = res.expect("could not unwrap graph").clone();
                let graph: Result<CoreContainerTree> = Ok(graph.into());
                serde_json::to_value(graph).expect("could not convert graph into JsValue")
            }

            GraphCommand::Get(root) => {
                let Some(graph) = self.store.get_container_graph(&root) else {
                    let res: Result<Option<CoreResourceTree<CoreContainer>>> = Ok(None);
                    return serde_json::to_value(res)
                        .expect("could not convert to JsValue");
                };

                let dup = graph.duplicate(&root);
                let Ok(graph) = dup else {
                    let err = Error::LocalError("could not duplicate tree".into());
                    return serde_json::to_value(err).expect("could not convert error to JsValue");
                };

                let graph: CoreResourceTree<CoreContainer> = graph.into();
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

                let child: Result<CoreContainer> = Ok(child.clone().into());
                serde_json::to_value(child).expect("could not convert child `Container` to JsValue")
            }

            GraphCommand::Duplicate(root) => {
                let res = self.duplicate_container_tree(&root);
                let Ok(rid) = res else {
                    return serde_json::to_value(res).expect("could not convert `Result` to JsValue");
                };

                let Some(graph) = self.store.get_container_graph(&rid) else {
                    let err: Error =  CoreError::ResourceError(ResourceError::DoesNotExist("graph not found")).into();
                    return serde_json::to_value(err).expect("could not convert error to JsValue");
                };

                let dup = graph.duplicate(&rid);
                let Ok(dup) = dup else {
                    let err = Error::LocalError("could not duplicate tree".into());
                    return serde_json::to_value(err).expect("could not convert error to JsValue");
                };

                let dup: CoreResourceTree<CoreContainer> = dup.into();
                serde_json::to_value(dup).expect("could not convert `Result` to JsValue")
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
            let graph = ContainerTree::load(data_root)?;
            self.store.insert_project_graph(pid.clone(), graph);
        }

        let Some(graph) = self.store.get_project_graph(pid) else {
            return Err(Error::LocalError("could not load `Project` graph".to_string()));
        };

        Ok(graph)
    }

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

        let Some(parent) = graph.parent(rid)? else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not have parent")).into());
        };

        let mut dup = graph.duplicate(rid)?;
        let dup_id = dup.root().clone();
        drop(graph);

        dup.set_base_path(&dup_id, unique_file_name(root.base_path()?)?)?;
        let res = self
            .store
            .insert_subgraph(&parent.clone(), dup.into_inner());

        if let Err(err) = res {
            return Err(err);
        }

        Ok(dup_id)
    }

    fn new_child(&mut self, parent: &ResourceId, name: String) -> Result<ResourceId> {
        let Some(parent) = self.store.get_container(&parent) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist")).into());
        };

        // create child
        // @todo: Ensure unique and valid path.
        let child_path = unique_file_name(parent.base_path()?.join(&name))?;
        let cid = container::new(&child_path)?;
        let mut child = Container::load(&child_path)?;
        child.properties.name = Some(name);
        child.save()?;

        // insert into graph
        let child = ContainerTree::new(child);
        self.store
            .insert_subgraph(&parent.rid.clone(), child.into_inner())?;
        Ok(cid)
    }
}

#[cfg(test)]
#[path = "./graph_test.rs"]
mod graph_test;
