//! Local [`ResourceTree`](CoreTree).
use crate::project::container;
use crate::project::resources::Container;
use crate::Result;
use has_id::HasId;
use settings_manager::LocalSettings;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::graph::{tree::NodeMap, ResourceNode, ResourceTree as CoreResourceTree};
use thot_core::project::Container as CoreContainer;
use thot_core::types::ResourceId;

#[derive(Clone)]
pub struct ResourceTree<D>(CoreResourceTree<D>)
where
    D: HasId<Id = ResourceId> + LocalSettings;

impl<D> ResourceTree<D>
where
    D: HasId<Id = ResourceId> + LocalSettings,
{
    /// Create a new tree.
    pub fn new(root: D) -> Self {
        Self(CoreResourceTree::new(root))
    }

    /// Returns the inner [tree](CoreTree), consuming self.
    pub fn into_inner(self) -> CoreResourceTree<D> {
        self.0
    }
}

impl<D> Deref for ResourceTree<D>
where
    D: HasId<Id = ResourceId> + LocalSettings,
{
    type Target = CoreResourceTree<D>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> DerefMut for ResourceTree<D>
where
    D: HasId<Id = ResourceId> + LocalSettings,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// **********************
// *** container tree ***
// **********************

impl ResourceTree<Container> {
    /// Load a `Container` tree into a [`ResourceTree`](CoreTree).
    pub fn load(path: &Path) -> Result<Self> {
        let root = Container::load_or_default(&path)?;
        let rid = root.id().clone();
        let mut graph = CoreResourceTree::new(root);

        for entry in fs::read_dir(path)? {
            let dir = entry?;
            if container::path_is_container(&dir.path()) {
                let c_tree = Self::load(&dir.path())?;
                graph.insert_tree(&rid, c_tree.into_inner())?;
            }
        }

        Ok(Self(graph))
    }

    /// Duplicates a subtree.
    /// Base paths of the [`Containers`] are maintained.
    pub fn duplicate(&self, root: &ResourceId) -> Result<Self> {
        let Some(node) = self.get(root) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist in graph")).into());
        };

        let node = node.duplicate_with_path()?;
        let dup_root = node.rid.clone();
        let mut graph = Self(CoreResourceTree::new(node));
        let Some(children) = self.children(root) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` does not exist in graph")).into());
        };

        for child in children {
            let c_tree = self.duplicate(child)?;
            graph.insert_tree(&dup_root, c_tree.into_inner())?;
        }

        Ok(graph)
    }

    /// Sets the base path for `Container`.
    /// Updates paths in the subtree.
    pub fn set_base_path(&mut self, root: &ResourceId, path: PathBuf) -> Result {
        /// recursively sets base paths
        fn set_base_path_recursive(
            graph: &mut ResourceTree<Container>,
            root: &ResourceId,
            path: PathBuf,
        ) -> Result {
            let node = graph.get_mut(root).expect("`Container` not in graph");
            node.set_base_path(path)?;
            let path = node.base_path()?;

            for cid in graph
                .children(root)
                .expect("`Container` not in graph")
                .clone()
            {
                let child = graph.get(&cid).expect("child `Container` not in graph");
                let c_name = child.base_path()?;
                let c_name = c_name
                    .file_name()
                    .expect("could not parse child `Container`'s path");

                let mut c_path = path.clone();
                c_path.push(c_name);

                set_base_path_recursive(graph, &cid, c_path)?;
            }

            Ok(())
        }

        // set base paths
        Ok(set_base_path_recursive(self, root, path)?)
    }
}

impl Into<CoreResourceTree<CoreContainer>> for ResourceTree<Container> {
    fn into(self) -> CoreResourceTree<CoreContainer> {
        let (nodes, edges) = self.into_inner().into_components();
        let nodes = nodes
            .into_iter()
            .map(|(id, node)| {
                let container = node.into_data();
                let container: CoreContainer = container.into();
                (id, ResourceNode::new(container))
            })
            .collect::<NodeMap<CoreContainer>>();

        CoreResourceTree::from_components(nodes, edges).expect("could not convert tree")
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;
