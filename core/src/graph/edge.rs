//! Graph edges.
use super::NodeId;

// ***************
// *** Builder ***
// ***************

// -------------------
// --- state types ---
// -------------------

/// A node place holder.
#[derive(Default)]
pub struct NoNode;

/// A set node.
pub struct Node(NodeId);

// ---------------
// --- builder ---
// ---------------

/// Creates a directed edge.
#[derive(Default)]
pub struct DirectedEdgeBuilder<F, T> {
    from: F,
    to: T,
}

impl DirectedEdgeBuilder<NoNode, NoNode> {
    pub fn new() -> Self {
        DirectedEdgeBuilder::default()
    }
}

impl DirectedEdgeBuilder<Node, Node> {
    pub fn build(self) -> DirectedEdge {
        DirectedEdge {
            from: self.from.0,
            to: self.to.0,
        }
    }
}

impl<F, T> DirectedEdgeBuilder<F, T> {
    pub fn from(self, id: NodeId) -> DirectedEdgeBuilder<Node, T> {
        DirectedEdgeBuilder {
            from: Node(id),
            to: self.to,
        }
    }

    pub fn clear_from(self) -> DirectedEdgeBuilder<NoNode, T> {
        DirectedEdgeBuilder {
            from: NoNode,
            to: self.to,
        }
    }

    pub fn to(self, id: NodeId) -> DirectedEdgeBuilder<F, Node> {
        DirectedEdgeBuilder {
            from: self.from,
            to: Node(id),
        }
    }

    pub fn clear_to(self) -> DirectedEdgeBuilder<F, NoNode> {
        DirectedEdgeBuilder {
            from: self.from,
            to: NoNode,
        }
    }
}

// ********************
// *** DirectedEdge ***
// ********************

/// Connects graph nodes in a single direction.
pub struct DirectedEdge {
    /// Starting node of the edge.
    from: NodeId,

    /// Ending node of the edge.
    to: NodeId,
}

impl DirectedEdge {
    pub fn from(&self) -> &NodeId {
        &self.from
    }

    pub fn to(&self) -> &NodeId {
        &self.to
    }
}

#[cfg(test)]
#[path = "./edge_test.rs"]
mod edge_test;
