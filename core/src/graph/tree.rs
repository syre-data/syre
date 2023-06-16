//! A tree graph
use super::ResourceNode;
use crate::error::{GraphError, ResourceError};
use crate::types::{ResourceId, ResourceMap};
use crate::Result;
use has_id::HasId;
use indexmap::IndexSet;
use std::collections::hash_map::{Iter, IterMut};
use std::collections::HashSet;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// *************
// *** types ***
// *************

/// Map from node id to node.
pub type NodeMap<D> = ResourceMap<ResourceNode<D>>;

/// Map from parent node to children.
pub type EdgeMap = ResourceMap<IndexSet<ResourceId>>;

// *********************
// *** resource tree ***
// *********************

/// A tree graph.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub struct ResourceTree<D>
where
    D: HasId<Id = ResourceId>,
{
    root: ResourceId,
    nodes: NodeMap<D>,
    edges: EdgeMap,

    /// Map from children to their parent's id.
    parents: ResourceMap<Option<ResourceId>>,
}

impl<D> ResourceTree<D>
where
    D: HasId<Id = ResourceId>,
{
    pub fn new(root: D) -> Self {
        let mut nodes = NodeMap::new();
        let node = ResourceNode::new(root);
        let root = node.id().clone();
        nodes.insert(root.clone(), node);

        let mut edges = EdgeMap::new();
        edges.insert(root.clone(), IndexSet::new());

        let mut parents = ResourceMap::new();
        parents.insert(root.clone(), None);

        Self {
            root,
            nodes,
            edges,
            parents,
        }
    }

    pub fn from_components(nodes: NodeMap<D>, edges: EdgeMap) -> Result<Self> {
        let mut parents = ResourceMap::new();
        let mut root = nodes
            .keys()
            .map(|id| id.clone())
            .collect::<HashSet<ResourceId>>();

        // compute parents, find root.
        for (id, node) in nodes.iter() {
            let Some(children) = edges.get(&id) else {
                return Err(GraphError::InvalidGraph("node does not have edge map").into());
            };

            for child in children {
                parents.insert(child.clone(), Some(node.id().clone()));
                root.remove(child);
            }
        }

        if root.len() != 1 {
            return Err(GraphError::InvalidGraph("root `Node` not found").into());
        }

        let Some(root) = root.into_iter().next() else {
            return Err(GraphError::InvalidGraph("could not get root").into());
        };

        parents.insert(root.clone(), None);

        Ok(Self {
            root,
            nodes,
            edges,
            parents,
        })
    }

    /// Get the id of the root of the tree.
    pub fn root(&self) -> &ResourceId {
        &self.root
    }

    pub fn nodes(&self) -> &NodeMap<D> {
        &self.nodes
    }

    pub fn edges(&self) -> &EdgeMap {
        &self.edges
    }

    /// Returns an iterator over the graph's nodes.
    pub fn iter_nodes(&self) -> Iter<ResourceId, ResourceNode<D>> {
        self.nodes.iter()
    }

    /// Returns a `mut`able iterator over the graph's nodes.
    pub fn iter_nodes_mut(&mut self) -> IterMut<ResourceId, ResourceNode<D>> {
        self.nodes.iter_mut()
    }

    /// Get a [`Node`] by its id.
    pub fn get(&self, id: &ResourceId) -> Option<&ResourceNode<D>> {
        self.nodes.get(&id)
    }

    /// Get a `mut`able [`Node`] by its id.
    pub fn get_mut(&mut self, id: &ResourceId) -> Option<&mut ResourceNode<D>> {
        self.nodes.get_mut(&id)
    }

    /// Inserts a new node into the tree.
    ///
    /// # Arguments
    /// 1. Parent id.
    /// 2. Node data.
    ///
    /// # Errors
    /// + [`ResourceError`] if the `parent` node does not exist.
    pub fn insert(&mut self, parent: ResourceId, data: D) -> Result {
        // check parent exists
        if !self.nodes.contains_key(&parent) {
            return Err(ResourceError::DoesNotExist("parent `Node` not found").into());
        }

        let Some(children) = self.edges.get_mut(&parent) else {
            return Err(ResourceError::DoesNotExist("parent `Node` not found").into());
        };

        let node = ResourceNode::new(data);
        let id = node.id().clone();

        self.nodes.insert(id.clone(), node);
        children.insert(id.clone());
        self.edges.insert(id.clone(), IndexSet::new());
        self.parents.insert(id.clone(), Some(parent.clone()));

        Ok(())
    }

    /// Returns the children `Containers` of the parent, if found.
    pub fn children(&self, parent: &ResourceId) -> Option<&IndexSet<ResourceId>> {
        self.edges.get(&parent).to_owned()
    }

    /// Returns the parent of a `Node`.
    ///
    /// # Errors
    /// + If the child does not exist.
    pub fn parent(&self, child: &ResourceId) -> Result<Option<&ResourceId>> {
        let Some(parent) = self.parents.get(&child) else {
            return Err(ResourceError::DoesNotExist("`Node` not found").into());
        };

        Ok(parent.as_ref())
    }

    /// Returns the path of ancesetors to the tree root.
    /// Begins with self.
    ///
    /// # Returns
    /// + Empty `Vec` if the `root` `Node` is not found.
    /// + `Vec` of the ancestor path to the tree root, beginning with self.
    pub fn ancestors(&self, root: &ResourceId) -> Vec<ResourceId> {
        let mut ancestors = Vec::new();
        let mut current = self.get(root).map(|node| node.id());
        while let Some(id) = current {
            ancestors.push(id.clone());
            current = self.parent(id).expect("parent not found");
        }

        return ancestors;
    }

    /// Returns all the descendants of the root.
    ///
    /// # Returns
    /// + `None` if the root `Node` does not exist.
    /// + Descendant Ids, otherwise.
    pub fn descendants(&self, root: &ResourceId) -> Option<HashSet<ResourceId>> {
        let mut descendants = HashSet::new();
        if !self.get(&root).is_some() {
            return None;
        }

        descendants.insert(root.clone());

        let Some(children) = self.children(root) else {
            return Some(descendants);
        };

        for child in children {
            if let Some(c_descendants) = self.descendants(child) {
                for did in c_descendants {
                    descendants.insert(did);
                }
            }
        }

        return Some(descendants);
    }

    /// Inserts a [`Tree`] as a subtree.
    pub fn insert_tree(&mut self, parent: &ResourceId, tree: Self) -> Result {
        // insert root
        let Some(p_edges) = self.edges.get_mut(&parent) else {
            return Err(ResourceError::DoesNotExist("parent edges not found").into());
        };

        let root = tree.root().clone();
        p_edges.insert(root.clone());
        self.parents.insert(root, Some(parent.clone()));

        // insert tree
        let (nodes, edges) = tree.into_components();

        for (id, node) in nodes.into_iter() {
            self.nodes.insert(id, node);
        }

        for (parent, children) in edges.into_iter() {
            for child in children.clone() {
                self.parents.insert(child, Some(parent.clone()));
            }

            self.edges.insert(parent, children);
        }

        Ok(())
    }

    /// Removes a subtree.
    ///
    /// # Returns
    /// The removed subtree.
    pub fn remove(&mut self, root: &ResourceId) -> Result<Self> {
        let parent = self
            .parent(root)
            .expect("root `Node` not found")
            .expect("root `Node` can not be removed")
            .clone();

        let (nodes, edges) = self.remove_components(root)?;

        let p_edges = self
            .edges
            .get_mut(&parent)
            .expect("parent `Node` edges not found");

        p_edges.remove(root);

        let mut parents = ResourceMap::new();
        for (parent, children) in edges.clone() {
            for child in children {
                parents.insert(child.clone(), Some(parent.clone()));
            }
        }

        Ok(Self {
            root: root.clone(),
            nodes,
            edges,
            parents,
        })
    }

    /// Moves a subtree to a different parent.
    ///
    /// # Errors
    /// + If root `Node` does not exist.
    /// + If the new parent does not exist.
    pub fn mv(&mut self, root: &ResourceId, parent: &ResourceId) -> Result {
        // remove from original parent
        let Some(Some(o_parent)) = self.parents.get(&root) else {
            return Err(ResourceError::DoesNotExist("parent `Node` does not exist").into());
        };

        let Some(op_edges) = self.edges.get_mut(o_parent) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist").into());
        };

        op_edges.remove(root);

        // add to new parent
        let Some(np_edges) = self.edges.get_mut(parent) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist").into());
        };

        np_edges.insert(root.clone());
        self.parents.insert(root.clone(), Some(parent.clone()));

        Ok(())
    }

    /// Sets the index of a `Node` amongst its parent's children.
    ///
    /// # See also
    /// + Follows the rules of [`indexset::IndexSet::move_index`].
    pub fn move_index(&mut self, node: &ResourceId, index: usize) -> Result {
        let Some(Some(parent)) = self.parents.get(&node) else {
            return Err(ResourceError::DoesNotExist("`Node` parent does not exist").into());
        };

        let Some(edges) = self.edges.get_mut(parent) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist").into());
        };

        let Some(curr_index) = edges.get_index_of(node) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist").into());
        };

        edges.move_index(curr_index, index);
        Ok(())
    }

    /// Consumes self, returning the graph's nodes and edges.
    pub fn into_components(self) -> (NodeMap<D>, EdgeMap) {
        (self.nodes, self.edges)
    }

    /// Recursively removes a subtree.
    fn remove_components(&mut self, root: &ResourceId) -> Result<(NodeMap<D>, EdgeMap)> {
        let mut nodes = NodeMap::new();
        let mut edges = EdgeMap::new();

        // remove root node
        let Some(node) = self.nodes.remove(root) else {
            return Err(ResourceError::DoesNotExist("`Node` does not exist").into());
        };

        self.parents.remove(node.id());
        nodes.insert(node.id().clone(), node);

        // remove children
        let Some(children) = self.edges.remove(&root) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist").into());
        };

        for child in children.iter() {
            let (c_nodes, c_edges) = self.remove_components(&child)?;

            for (c_id, c_node) in c_nodes.into_iter() {
                nodes.insert(c_id, c_node);
            }

            for (parent, p_edges) in c_edges.into_iter() {
                edges.insert(parent, p_edges);
            }
        }

        edges.insert(root.clone(), children);

        Ok((nodes, edges))
    }
}

impl<D> ResourceTree<D>
where
    D: HasId<Id = ResourceId> + Clone,
{
    /// Clones a subtree.
    pub fn clone_tree(&self, root: &ResourceId) -> Result<Self> {
        let Some(root_node) = self.nodes.get(&root) else {
            return Err(ResourceError::DoesNotExist("root `Node` not found").into());
        };

        let mut tree = Self::new(root_node.clone().into_data());
        for child in self.children(root).expect("children not found") {
            tree.insert_tree(root, self.clone_tree(child)?)?;
        }

        Ok(tree)
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;
