//! Graph structures.
pub mod edge;
pub mod graph;
pub mod node;
pub mod tree;

// Re-exports
pub use edge::{DirectedEdge, DirectedEdgeBuilder};
pub use graph::Graph;
pub use node::{Node, NodeId};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
