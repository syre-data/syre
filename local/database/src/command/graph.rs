//! Container related commands.
use serde::{Deserialize, Serialize};
use syre_core::types::ResourceId;

/// Graph related commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum GraphCommand {
    /// Loads a `Project`'s graph.
    /// Reloads it if is already loaded.
    ///
    /// # Fields
    /// 0. Id of the project.
    Load(ResourceId),

    /// Gets a `Project`'s graph, loading it if needed.
    ///
    /// # Fields
    /// 0. Id of the project.
    GetOrLoad(ResourceId),

    /// Gets a subtree.
    ///
    /// # Fields
    /// 0. Id of the root of the desired subtree.
    Get(ResourceId),

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
