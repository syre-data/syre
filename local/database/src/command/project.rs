//! Project related commands.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::project::Project as CoreProject;
use syre_core::types::ResourceId;

/// Project related commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum ProjectCommand {
    /// Load a [`Project`](syre_local::project::resources::Project) from a path.
    ///
    /// # Returns
    /// [`Project`](syre_local::project::resources::Project).
    Load(PathBuf),

    /// Load a [`Project`](syre_local::project::resources::Project) from a path.
    ///
    /// # Returns
    /// Tuples of ([`Project`](syre_local::project::resources::Project), [`ProjectSettings`][`Project`](syre_local::project::resources::ProjectSettings)).
    LoadWithSettings(PathBuf),

    /// Loads the user's projects.
    ///
    /// # Fields
    /// 1. User's id.
    ///
    /// # Returns
    /// A tuple of loaded projects and projects that errored while loading,
    LoadUser(ResourceId),

    /// Retrieves a [`Project`](syre_core::project::Project) by [`ResourceId`].
    Get(ResourceId),

    /// Update a [`Project`](CoreProject).
    Update(CoreProject),

    /// Gets the path to the `Project`.
    GetPath(ResourceId),

    /// Get's the project root path of a resource given its path.
    ResourceRootPath(PathBuf),
}
