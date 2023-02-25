//! A graph
use super::{Node, NodeId};
use std::collections::{HashMap, HashSet};

/// A graph structure.
pub struct Graph<D, E> {
    nodes: HashMap<NodeId, Node<D>>,
    edges: HashSet<E>,
}

#[cfg(test)]
#[path = "./graph_test.rs"]
mod graph_test;
