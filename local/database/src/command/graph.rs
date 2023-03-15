//! Container related commands.
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

/// Graph related commands.
#[derive(Serialize, Deserialize)]
pub enum GraphCommand {
    /// Load a `Project`'s graph.
    Load(ResourceId),

    /// Gets a subtree.
    Get(ResourceId),

    /// Remove a graph from its root.
    Remove(ResourceId),

    /// Insert a child into the graph.
    NewChild(NewChildArgs),

    /// Duplicate a graph from its root.
    Duplicate(ResourceId),
}

/// Arguments for [`Command::NewChild`].
#[derive(Serialize, Deserialize)]
pub struct NewChildArgs {
    pub name: String,
    pub parent: ResourceId,
}

#[cfg(test)]
#[path = "./graph_test.rs"]
mod graph_test;
