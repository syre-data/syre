//! A tree graph
use super::DirectedEdge;
use super::{Node, NodeId};
use crate::error::ResourceError;
use crate::Result;
use has_id::HasId;
use std::collections::{HashMap, HashSet};

/// A tree graph.
pub struct Tree<D> {
    root: NodeId,
    nodes: HashMap<NodeId, Node<D>>,
    edges: HashSet<DirectedEdge>,
}

impl<D> Tree<D> {
    pub fn new(root: D) -> Self {
        let mut nodes = HashMap::new();
        let node = Node::new(root);
        let root = node.id().clone();
        nodes.insert(root.clone(), node);

        Self {
            root,
            nodes,
            edges: HashSet::default(),
        }
    }

    /// Get the id of the root of the tree.
    pub fn root(&self) -> &NodeId {
        &self.root
    }

    /// Get a [`Node`] by its id.
    pub fn get(&self, id: &NodeId) -> Option<&Node<D>> {
        self.nodes.get(&id)
    }

    /// Get a `mut`able [`Node`] by its id.
    pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node<D>> {
        self.nodes.get_mut(&id)
    }

    /// Inserts a new node into the tree.
    ///
    /// # Arguments
    /// 1. Node data.
    /// 2. Parent id.
    ///
    /// # Returns
    /// Id of the new [`Node`].
    ///
    /// # Errors
    /// + [`ResourceError`] if the `parent` node does not exist.
    pub fn insert(&mut self, data: D, parent: NodeId) -> Result<NodeId> {
        // check parent exists
        if !self.nodes.contains_key(&parent) {
            return Err(ResourceError::DoesNotExist("parent `Node` not found".to_string()).into());
        }

        let node = Node::new(data);
        let id = node.id().clone();

        self.nodes.insert(id.clone(), node);
        Ok(id)
    }

    pub fn children(&self, parent: &NodeId) -> Option<HashSet<NodeId>> {
        todo!();
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;
