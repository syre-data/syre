//! Container related commands.
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

/// Graph related commands.
#[derive(Serialize, Deserialize, Debug)]
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

    /// Get the children of the Container.
    Children(ResourceId),

    /// Get the parent of the Container.
    Parent(ResourceId),
}

/// Arguments for [`Command::NewChild`].
#[derive(Serialize, Deserialize, Debug)]
pub struct NewChildArgs {
    pub name: String,
    pub parent: ResourceId,
}
