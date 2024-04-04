//! Implementation of graph related functionality.
use super::super::Database;
use crate::command::GraphCommand;
use crate::error::server::LoadProjectGraph as LoadProjectGraphError;
use crate::server::store::{object_store, ContainerTree as LocalContainerTree};
use crate::Result;
use serde_json::Value as JsValue;
use std::result::Result as StdResult;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::graph::ResourceTree;
use syre_core::project::Container as CoreContainer;
use syre_core::types::ResourceId;
use syre_local::common::unique_file_name;
use syre_local::graph::{ContainerTreeDuplicator, ContainerTreeTransformer};
use syre_local::loader::container::Loader as ContainerLoader;
use syre_local::loader::tree::incremental::{Loader as ContainerTreeLoader, PartialLoad};
use syre_local::project::container;

type ContainerTree = ResourceTree<CoreContainer>;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_graph(&mut self, cmd: GraphCommand) -> JsValue {
        match cmd {
            GraphCommand::Load(project) => {
                let res = self.handle_load_project_graph(&project);
                serde_json::to_value(res).unwrap()
            }

            GraphCommand::GetOrLoad(project) => {
                let res = match self.object_store.get_project_graph(&project) {
                    Some(graph) => Ok(ContainerTreeTransformer::local_to_core(graph)),
                    None => self.handle_load_project_graph(&project),
                };

                serde_json::to_value(res).unwrap()
            }

            GraphCommand::Get(root) => {
                let graph = self.handle_graph_get(&root);
                serde_json::to_value(graph).unwrap()
            }

            GraphCommand::Duplicate(root) => {
                let graph = self.handle_graph_duplicate_tree(&root);
                serde_json::to_value(graph).unwrap()
            }

            GraphCommand::Parent(rid) => {
                serde_json::to_value(self.handle_graph_parent(&rid)).unwrap()
            }

            GraphCommand::Children(rid) => {
                let children = self.handle_graph_children(&rid);
                serde_json::to_value(children).unwrap()
            }
        }
    }

    /// Convenience function to handle loading a project graph and its errors.
    fn handle_load_project_graph(
        &mut self,
        project: &ResourceId,
    ) -> StdResult<ContainerTree, LoadProjectGraphError> {
        match self.load_project_graph(project) {
            Ok(_) => {
                let graph = self.object_store.get_project_graph(project).unwrap();
                let graph = ContainerTreeTransformer::local_to_core(graph);

                if let Err(err) = self
                    .data_store
                    .graph()
                    .create(graph.clone(), project.clone())
                {
                    tracing::error!("could not add graph to data store: {err:?}");
                }

                Ok(graph)
            }

            Err(error::LoadProjectGraph_Local::ProjectNotFound) => {
                tracing::error!("project not found");
                Err(LoadProjectGraphError::ProjectNotFound)
            }

            Err(error::LoadProjectGraph_Local::Project(err)) => {
                tracing::error!(?err);
                Err(LoadProjectGraphError::Project(err))
            }

            Err(error::LoadProjectGraph_Local::Load(PartialLoad { errors, graph })) => {
                tracing::error!(?errors);
                let graph = graph.map(|graph| ContainerTreeTransformer::local_to_core(&graph));
                Err(LoadProjectGraphError::Load { errors, graph })
            }

            Err(error::LoadProjectGraph_Local::InsertContainers(errors)) => {
                tracing::error!(?errors);
                Err(LoadProjectGraphError::InsertContainers(errors.into()))
            }

            Err(error::LoadProjectGraph_Local::InsertAssets(
                object_store::error::AssetsGraph {
                    assets: errors,
                    graph: _,
                },
            )) => {
                tracing::error!(?errors);
                let graph = self.object_store.get_project_graph(&project).unwrap();
                let graph = ContainerTreeTransformer::local_to_core(graph);
                Err(LoadProjectGraphError::InsertAssets {
                    errors: errors.into(),
                    graph,
                })
            }
        }
    }

    /// Loads a `Project`'s [`Container`](LocalContainer) tree from settings.
    fn load_project_graph(
        &mut self,
        pid: &ResourceId,
    ) -> StdResult<(), error::LoadProjectGraph_Local> {
        let Some(project) = self.object_store.get_project(pid) else {
            return Err(error::LoadProjectGraph_Local::ProjectNotFound);
        };

        let graph = match ContainerTreeLoader::load(project.data_root_path()) {
            Ok(graph) => graph,
            Err(PartialLoad { errors, graph }) => {
                return Err(PartialLoad { errors, graph }.into());
            }
        };

        self.object_store.remove_project_graph(pid);
        match self.object_store.insert_project_graph(pid.clone(), graph) {
            Ok(_old_graph) => {}
            Err(err) => return Err(err.into()),
        }

        Ok(())
    }

    fn handle_graph_get(&self, root: &ResourceId) -> Option<ResourceTree<CoreContainer>> {
        self.object_store
            .get_graph_of_container(&root)
            .map(|graph| ContainerTreeTransformer::local_to_core(graph))
    }

    fn handle_graph_duplicate_tree(
        &mut self,
        root: &ResourceId,
    ) -> Result<ResourceTree<CoreContainer>> {
        self.duplicate_container_tree(&root)?;
        let graph = self.object_store.get_graph_of_container(&root).unwrap();
        Ok(ContainerTreeTransformer::subtree_to_core(graph, &root).unwrap())
    }

    // TODO: Should not write to file system
    //      but only copy properties in memory
    //      and leave to client to modify file system.
    //      May be better to remove entirely.
    //      Client would get a subtree, duplicate it themselves, and modify the file system.
    /// Duplicates a tree in its parent.
    /// Returns the id of the duplicated tree's root node.
    #[tracing::instrument(skip(self))]
    fn duplicate_container_tree(&mut self, rid: &ResourceId) -> Result<ResourceId> {
        let Some(project) = self.object_store.get_container_project(rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` `Project` not loaded",
            ))
            .into());
        };

        let Some(graph) = self.object_store.get_project_graph(&project) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Project` graph not loaded",
            ))
            .into());
        };

        let Some(root) = graph.get(rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` does not exist in graph",
            ))
            .into());
        };

        let Some(parent) = graph.parent(rid).unwrap().cloned() else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` does not have parent",
            ))
            .into());
        };

        // duplicate tree
        let dup_path = unique_file_name(root.base_path())?;
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
        let res = self.object_store.insert_subgraph(&parent, dup);
        match res {
            Ok(_) => Ok(dup_root),
            Err(err) => Err(err),
        }
    }

    fn new_child(&mut self, parent: &ResourceId, name: String) -> Result<ResourceId> {
        let Some(parent) = self.object_store.get_container(&parent) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` does not exist",
            ))
            .into());
        };

        // create child
        // TODO Ensure unique and valid path.
        let child_path = unique_file_name(parent.base_path().join(&name))?;
        let cid = container::new(&child_path)?;
        let mut child = ContainerLoader::load(child_path)?;
        child.properties.name = name;
        child.save()?;

        // insert into graph
        let child = LocalContainerTree::new(child);
        self.object_store
            .insert_subgraph(&parent.rid.clone(), child)?;

        Ok(cid)
    }

    fn handle_graph_parent(
        &self,
        child: &ResourceId,
    ) -> StdResult<Option<&ResourceId>, ResourceError> {
        let Some(graph) = self.object_store.get_graph_of_container(&child) else {
            return Err(ResourceError::does_not_exist("`Container` does not exist"));
        };

        Ok(graph.parent(&child).unwrap())
    }

    fn handle_graph_children(
        &self,
        parent: &ResourceId,
    ) -> StdResult<&indexmap::IndexSet<ResourceId>, ResourceError> {
        let Some(graph) = self.object_store.get_graph_of_container(parent) else {
            return Err(ResourceError::does_not_exist("`Container` does not exist"));
        };

        Ok(graph.children(parent).unwrap())
    }
}

pub mod error {
    use crate::server::store::object_store::{self, error::InsertProjectGraph};
    use std::collections::HashMap;
    use std::io;
    use std::path::PathBuf;
    use syre_core::error::Project;
    use syre_local::loader::tree::incremental::PartialLoad;
    use thiserror::Error;

    /// Used for errors local to this module.
    #[allow(non_camel_case_types)]
    #[derive(Error, Debug)]
    pub(super) enum LoadProjectGraph_Local {
        #[error("project not found")]
        ProjectNotFound,

        #[error("{0:?}")]
        Project(Project),

        #[error("{0:?}")]
        Load(PartialLoad),

        #[error("{0:?}")]
        InsertContainers(HashMap<PathBuf, io::ErrorKind>),

        #[error("{0:?}")]
        InsertAssets(object_store::error::AssetsGraph),
    }

    impl From<Project> for LoadProjectGraph_Local {
        fn from(value: Project) -> Self {
            Self::Project(value)
        }
    }

    impl From<PartialLoad> for LoadProjectGraph_Local {
        fn from(value: PartialLoad) -> Self {
            Self::Load(value)
        }
    }

    impl From<InsertProjectGraph> for LoadProjectGraph_Local {
        fn from(value: InsertProjectGraph) -> Self {
            match value {
                InsertProjectGraph::Tree(errors) => Self::InsertContainers(errors),
                InsertProjectGraph::Assets(err) => Self::InsertAssets(err),
            }
        }
    }
}
