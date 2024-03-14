//! Implementation of graph related functionality.
use super::super::Database;
use crate::command::GraphCommand;
use crate::error::server::LoadProjectGraph;
use crate::server::store;
use crate::server::store::ContainerTree;
use crate::{Error, Result};
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

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_graph(&mut self, cmd: GraphCommand) -> JsValue {
        match cmd {
            GraphCommand::Load(project) => self.handle_load_project_graph(&project),

            GraphCommand::GetOrLoad(project) => {
                if let Some(graph) = self.store.get_project_graph(&project) {
                    let graph = ContainerTreeTransformer::local_to_core(graph);
                    let res: Result<ResourceTree<CoreContainer>> = Ok(graph);
                    return serde_json::to_value(res).unwrap();
                }
                self.handle_load_project_graph(&project)
            }

            GraphCommand::Get(root) => {
                let Some(graph) = self.store.get_graph_of_container(&root) else {
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
                let Some(graph) = self.store.get_graph_of_container(&rid) else {
                    let err: Result<ResourceTree<CoreContainer>> = Err(CoreError::Resource(
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
                let Some(graph) = self.store.get_graph_of_container(&rid) else {
                    let err: Result<Option<ResourceId>> = Err(Error::Core(CoreError::Resource(
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
                let Some(graph) = self.store.get_graph_of_container(&rid) else {
                    let res: Option<indexmap::IndexSet<ResourceId>> = None;
                    return serde_json::to_value(res)
                        .expect("could not convert `Container` to JsValue");
                };

                let children = graph.children(&rid).unwrap();
                serde_json::to_value(children).expect("could not convert `Container` to JsValue")
            }
        }
    }

    /// Convenience function to handle loading a project graph and its errors.
    fn handle_load_project_graph(&mut self, project: &ResourceId) -> JsValue {
        match self.load_project_graph(project) {
            Ok(graph) => {
                let graph = ContainerTreeTransformer::local_to_core(graph);
                let res: Result<ResourceTree<CoreContainer>> = Ok(graph);
                serde_json::to_value(res).unwrap()
            }

            Err(error::LoadProjectGraph_Local::ProjectNotFound) => {
                let err = StdResult::<ResourceTree<CoreContainer>, LoadProjectGraph>::Err(
                    LoadProjectGraph::ProjectNotFound,
                );

                serde_json::to_value(err).unwrap()
            }

            Err(error::LoadProjectGraph_Local::Project(err)) => {
                let err = StdResult::<ResourceTree<CoreContainer>, LoadProjectGraph>::Err(
                    LoadProjectGraph::Project(err),
                );

                serde_json::to_value(err).unwrap()
            }

            Err(error::LoadProjectGraph_Local::Load(PartialLoad { errors, graph })) => {
                let graph = graph.map(|graph| ContainerTreeTransformer::local_to_core(&graph));
                let err = StdResult::<ResourceTree<CoreContainer>, LoadProjectGraph>::Err(
                    LoadProjectGraph::Load { errors, graph },
                );

                serde_json::to_value(err).unwrap()
            }

            Err(error::LoadProjectGraph_Local::InsertContainers(errors)) => {
                let err = StdResult::<ResourceTree<CoreContainer>, LoadProjectGraph>::Err(
                    LoadProjectGraph::InsertContainers(errors.into()),
                );

                serde_json::to_value(err).unwrap()
            }

            Err(error::LoadProjectGraph_Local::InsertAssets(store::error::AssetsGraph {
                assets: errors,
                graph: _,
            })) => {
                let graph = self.store.get_project_graph(&project).unwrap();
                let graph = ContainerTreeTransformer::local_to_core(graph);
                let err = StdResult::<ResourceTree<CoreContainer>, LoadProjectGraph>::Err(
                    LoadProjectGraph::InsertAssets {
                        errors: errors.into(),
                        graph,
                    },
                );

                serde_json::to_value(err).unwrap()
            }
        }
    }

    /// Loads a `Project`'s [`Container`](LocalContainer) tree from settings.
    fn load_project_graph(
        &mut self,
        pid: &ResourceId,
    ) -> StdResult<&ContainerTree, error::LoadProjectGraph_Local> {
        let Some(project) = self.store.get_project(pid) else {
            return Err(error::LoadProjectGraph_Local::ProjectNotFound);
        };

        let graph = match ContainerTreeLoader::load(project.data_root_path()) {
            Ok(graph) => graph,
            Err(PartialLoad { errors, graph }) => {
                return Err(PartialLoad { errors, graph }.into());
            }
        };

        self.store.remove_project_graph(pid);
        match self.store.insert_project_graph(pid.clone(), graph) {
            Ok(_old_graph) => {}
            Err(err) => return Err(err.into()),
        }

        Ok(self.store.get_project_graph(pid).unwrap())
    }

    /// Duplicates a tree in its parent.
    /// Returns the id of the duplicated tree's root node.
    #[tracing::instrument(skip(self))]
    fn duplicate_container_tree(&mut self, rid: &ResourceId) -> Result<ResourceId> {
        let Some(project) = self.store.get_container_project(rid) else {
            return Err(CoreError::Resource(ResourceError::does_not_exist(
                "`Container` `Project` not loaded",
            ))
            .into());
        };

        let Some(graph) = self.store.get_project_graph(&project) else {
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

        let Some(parent) = graph.parent(rid)?.cloned() else {
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
        let res = self.store.insert_subgraph(&parent, dup);
        match res {
            Ok(_) => Ok(dup_root),
            Err(err) => Err(err),
        }
    }

    fn new_child(&mut self, parent: &ResourceId, name: String) -> Result<ResourceId> {
        let Some(parent) = self.store.get_container(&parent) else {
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
        let child = ContainerTree::new(child);
        self.store.insert_subgraph(&parent.rid.clone(), child)?;
        Ok(cid)
    }
}

pub mod error {
    use crate::server::store;
    use std::collections::HashMap;
    use std::io;
    use std::path::PathBuf;
    use store::error::InsertProjectGraph;
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
        InsertAssets(store::error::AssetsGraph),
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
