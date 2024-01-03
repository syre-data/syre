//! Local [`ResourceTree`](CoreTres).
use super::error::LoaderError;
use crate::common;
use crate::error::{LoaderErrors, Result};
use crate::project::container;
use crate::project::resources::Container;
use has_id::HasId;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::result::Result as StdResult;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::graph::tree::{EdgeMap, NodeMap};
use thot_core::graph::{ResourceNode, ResourceTree};
use thot_core::project::{Asset, Container as CoreContainer};
use thot_core::types::ResourceId;

type ContainerTree = ResourceTree<Container>;

pub struct ContainerTreeTransformer;
impl ContainerTreeTransformer {
    /// Convert a Container tree to a Core Container tree.
    pub fn local_to_core(tree: &ContainerTree) -> ResourceTree<CoreContainer> {
        let core_nodes = tree
            .nodes()
            .iter()
            .map(|(rid, node)| (rid.clone(), ResourceNode::new((*node.data()).clone())))
            .collect::<NodeMap<CoreContainer>>();

        ResourceTree::from_components(core_nodes, tree.edges().clone())
            .expect("could not build tree from components")
    }

    /// Convert a subtree of a Container tree to a Core Container tree.
    ///
    /// # Returns
    /// The converted subtree if the root node was found, otherwise `None`.
    pub fn subtree_to_core(
        tree: &ContainerTree,
        root: &ResourceId,
    ) -> Option<ResourceTree<CoreContainer>> {
        let Some(rids) = tree.descendants(root) else {
            return None;
        };

        let mut core_nodes = NodeMap::new();
        let mut edges = EdgeMap::new();
        for rid in rids {
            let node = tree.get(&rid).expect("descendant not it graph");
            core_nodes.insert(rid.clone(), ResourceNode::new((*node.data()).clone()));
            edges.insert(
                rid.clone(),
                tree.children(&rid)
                    .expect("could not get children of descendant")
                    .clone(),
            );
        }

        let graph =
            ResourceTree::from_components(core_nodes, edges).expect("could not reconstuct graph");

        Some(graph)
    }
}

// **************
// *** Loader ***
// **************

pub struct ContainerTreeLoader;
impl ContainerTreeLoader {
    /// Load a `Container` tree into a [`ResourceTree`].
    pub fn load(path: impl AsRef<Path>) -> StdResult<ContainerTree, LoaderErrors> {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(10)
            .build()
            .unwrap();
        let path = path.as_ref();
        return thread_pool.install(move || Self::load_tree(path));
    }

    fn load_tree(path: impl AsRef<Path>) -> StdResult<ContainerTree, LoaderErrors> {
        let path = path.as_ref();
        let root = match Container::load_from(path) {
            Ok(root) => root,
            Err(err) => return Err(vec![err.into()]),
        };

        let rid = root.id().clone();
        let mut graph = ResourceTree::new(root);

        let mut children = Vec::new();
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                return Err(vec![LoaderError::Io {
                    path: path.to_path_buf(),
                    kind: err.kind(),
                }])
            }
        };

        for entry in entries {
            let dir = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    return Err(vec![LoaderError::Io {
                        path: path.to_path_buf(),
                        kind: err.kind(),
                    }])
                }
            };

            if container::path_is_container(&dir.path()) {
                children.push(dir.path());
            }
        }

        let children = children
            .into_par_iter()
            .map(|path| Self::load_tree(path))
            .collect::<Vec<_>>();

        let mut errors = Vec::new();
        for child in children {
            match child {
                Ok(child) => graph.insert_tree(&rid, child).unwrap(),
                Err(mut err) => errors.append(&mut err),
            }
        }

        if errors.is_empty() {
            Ok(graph)
        } else {
            Err(errors)
        }
    }
}

// ******************
// *** Duplicator ***
// ******************

pub struct ContainerTreeDuplicator;
impl ContainerTreeDuplicator {
    /// Duplicates a subtree.
    ///
    /// # Arguments
    /// 1. Graph.
    /// 2. Id of the root of the subtree to duplicate.
    ///
    /// # Notes
    /// + `Asset`s are duplicated.
    #[tracing::instrument(skip(graph))]
    pub fn duplicate(graph: &ContainerTree, root: &ResourceId) -> thot_core::Result<ContainerTree> {
        // ensure root exists
        let Some(node) = graph.get(root) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist in graph",
            )));
        };

        // duplicate container to new location
        let mut container = Container::new(node.base_path());
        container.properties = node.properties.clone();
        container.scripts = node.scripts.clone();
        for asset_base in node.assets.values() {
            let mut asset = Asset::new(asset_base.path.clone());
            asset.properties = asset_base.properties.clone();
            container.insert_asset(asset);
        }

        let dup_root = container.rid.clone();
        let mut dup_graph = ResourceTree::new(container);
        let Some(children) = graph.children(&root).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist in graph",
            )));
        };

        for child in children {
            let c_tree = Self::duplicate(graph, &child)?;
            dup_graph.insert_tree(&dup_root, c_tree)?;
        }

        Ok(dup_graph)
    }

    /// Duplicates a subtree to a new file path.
    /// Base paths of the [`Containers`] are updated.
    ///
    /// # Arguments
    /// 1. Path to duplicate the tree to.
    /// 2. Graph.
    /// 3. Id of the root of the subtree to duplicate.
    ///
    /// # Notes
    /// + `Asset`s are not copied.
    #[tracing::instrument(skip(graph))]
    pub fn duplicate_without_assets_to(
        path: &Path,
        graph: &ContainerTree,
        root: &ResourceId,
    ) -> Result<ContainerTree> {
        // ensure root exists
        let Some(node) = graph.get(root) else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist in graph",
            ))
            .into());
        };

        // duplicate container to new location
        let mut container = Container::new(path);
        container.properties = node.properties.clone();
        container.scripts = node.scripts.clone();
        container.save()?;

        let dup_root = container.rid.clone();
        let mut dup_graph = ResourceTree::new(container);
        let Some(children) = graph.children(&root).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Container` does not exist in graph",
            ))
            .into());
        };

        for child in children {
            let rel_path = graph
                .get(&child)
                .expect("could not get child node")
                .base_path()
                .file_name()
                .expect("could not get name of `Container`");

            let mut c_path = path.to_path_buf();
            c_path.push(rel_path);
            let c_path = common::normalize_path_separators(c_path);

            let c_tree = Self::duplicate_without_assets_to(&c_path, graph, &child)?;
            dup_graph.insert_tree(&dup_root, c_tree)?;
        }

        Ok(dup_graph)
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;
