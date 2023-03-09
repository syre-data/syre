//! A tree graph
use super::ResourceNode;
use crate::error::ResourceError;
use crate::types::{ResourceId, ResourceMap};
use crate::Result;
use has_id::HasId;
use indexmap::IndexSet;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Map from node id to node.
type NodeMap<D> = ResourceMap<ResourceNode<D>>;

/// Map from parent node to children.
type EdgeMap = ResourceMap<IndexSet<ResourceId>>;

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

    /// Get the id of the root of the tree.
    pub fn root(&self) -> &ResourceId {
        &self.root
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
            return Err(ResourceError::DoesNotExist("parent `Node` not found".to_string()).into());
        }

        let Some(children) = self.edges.get_mut(&parent) else {
            return Err(ResourceError::DoesNotExist("parent `Node` not found".to_string()).into());
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

    /// Returns the parent of the child.
    ///
    /// # Errors
    /// + If the child does not exist.
    pub fn parent(&self, child: &ResourceId) -> Result<Option<&ResourceId>> {
        let Some(parent) = self.parents.get(&child) else {
            return Err(ResourceError::DoesNotExist("`Node` not found".to_string()).into());
        };

        Ok(parent.as_ref())
    }

    /// Inserts a [`Tree`] as a subtree.
    pub fn insert_tree(&mut self, parent: &ResourceId, tree: Self) -> Result {
        // insert root
        let Some(p_edges) = self.edges.get_mut(&parent) else {
            return Err(ResourceError::DoesNotExist("parent edges not found".to_string()).into());
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
        let (nodes, edges) = self.remove_components(root)?;
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
            return Err(ResourceError::DoesNotExist("parent `Node` does not exist".to_string()).into());
        };

        let Some(op_edges) = self.edges.get_mut(o_parent) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist".to_string()).into());
        };

        op_edges.remove(root);

        // add to new parent
        let Some(np_edges) = self.edges.get_mut(parent) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist".to_string()).into());
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
            return Err(ResourceError::DoesNotExist("`Node` parent does not exist".to_string()).into());
        };

        let Some(edges) = self.edges.get_mut(parent) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist".to_string()).into());
        };

        let Some(curr_index) = edges.get_index_of(node) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist".to_string()).into());
        };

        edges.move_index(curr_index, index);
        Ok(())
    }

    /// Consumes self, returning the graph's nodes and edges.
    fn into_components(self) -> (NodeMap<D>, EdgeMap) {
        (self.nodes, self.edges)
    }

    /// Recursively removes a subtree.
    fn remove_components(&mut self, root: &ResourceId) -> Result<(NodeMap<D>, EdgeMap)> {
        let mut nodes = NodeMap::new();
        let mut edges = EdgeMap::new();

        // remove root node
        let Some(node) = self.nodes.remove(root) else {
            return Err(ResourceError::DoesNotExist("`Node` does not exist".to_string()).into());
        };

        self.parents.remove(node.id());
        nodes.insert(node.id().clone(), node);

        // remove children
        let Some(children) = self.edges.remove(&root) else {
            return Err(ResourceError::DoesNotExist("`Node` edges do not exist".to_string()).into());
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

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;
