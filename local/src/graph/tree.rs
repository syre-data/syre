//! Local [`ResourceTree`](CoreTree).
use crate::project::container;
use crate::project::resources::container::{
    Builder as ContainerBuilder, Container, Loader as ContainerLoader,
};
use crate::Result;
use has_id::HasId;
use settings_manager::LocalSettings;
use std::fs;
use std::path::Path;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::types::ResourceId;

type ContainerTree = ResourceTree<Container>;

// **************
// *** Loader ***
// **************

pub struct ContainerTreeLoader;
impl ContainerTreeLoader {
    /// Load a `Container` tree into a [`ResourceTree`].
    pub fn load(path: &Path) -> Result<ContainerTree> {
        let root: Container = ContainerLoader::load_or_create(path.into())?.into();
        let rid = root.id().clone();
        let mut graph = ResourceTree::new(root);

        for entry in fs::read_dir(path)? {
            let dir = entry?;
            if container::path_is_container(&dir.path()) {
                let c_tree = Self::load(&dir.path())?;
                graph.insert_tree(&rid, c_tree)?;
            }
        }

        Ok(graph)
    }
}

// ******************
// *** Duplicator ***
// ******************

pub struct ContainerTreeDuplicator;
impl ContainerTreeDuplicator {
    /// Duplicates a subtree.
    /// Base paths of the [`Containers`] are maintained.
    ///
    /// # Arguments
    /// 1. Path to duplicate the tree to.
    /// 2. Graph.
    /// 3. Id of the root of the subtree to duplicate.
    pub fn duplicate_to(
        path: &Path,
        graph: &ContainerTree,
        root: &ResourceId,
    ) -> Result<ContainerTree> {
        // ensure root exists
        let Some(node) = graph.get(root) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist in graph")).into());
        };

        // duplicate container to new location
        let mut container = ContainerBuilder::default();
        let container_props = container.container_mut();
        *container_props = (*node).clone();
        container_props.rid = ResourceId::new();
        let container = container.save(path.into())?;

        let dup_root = container.rid.clone();
        let mut dup_graph = ResourceTree::new(container);
        let Some(children) = dup_graph.children(root).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist in graph")).into());
        };

        for child in children {
            let mut c_path = path.to_path_buf();
            c_path.push(
                node.base_path()
                    .file_name()
                    .expect("could not get file name of `Container`"),
            );

            let c_tree = Self::duplicate_to(&c_path, graph, &child)?;
            dup_graph.insert_tree(&dup_root, c_tree)?;
        }

        Ok(dup_graph)
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;
