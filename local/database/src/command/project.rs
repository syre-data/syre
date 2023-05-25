//! Project related commands.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_core::project::Project as CoreProject;
use thot_core::types::ResourceId;

/// Project related commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum ProjectCommand {
    /// Load a [`Project`](thot_local::project::resources::Project) from a path.
    ///
    /// # Returns
    /// [`Project`](thot_local::project::resources::Project).
    Load(PathBuf),

    /// Load a [`Project`](thot_local::project::resources::Project) from a path.
    ///
    /// # Returns
    /// Tuples of ([`Project`](thot_local::project::resources::Project), [`ProjectSettings`][`Project`](thot_local::project::resources::ProjectSettings)).
    LoadWithSettings(PathBuf),

    /// Adds a [`Project`](thot_local::project::resources::Project) for a user from a path.
    ///
    /// # Fields
    /// 1. Path to the project.
    /// 2. User.
    Add(PathBuf, ResourceId),

    /// Loads the user's projects.
    ///
    /// # Fields
    /// 1. User's id.
    ///
    /// # Returns
    /// A tuple of loaded projects and projects that errored while loading,
    LoadUser(ResourceId),

    /// Retrieves a [`Project`](thot_core::project::Project) by [`ResourceId`].
    Get(ResourceId),

    /// Update a [`Project`](CoreProject).
    Update(CoreProject),

    /// Gets the path to the `Project`.
    GetPath(ResourceId),

    /// Get's the project root path of a resource given its path.
    ResourceRootPath(PathBuf),
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
