//! Project related commands.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_core::project::Project as CoreProject;
use thot_core::types::ResourceId;

/// Project related commands.
#[derive(Serialize, Deserialize)]
pub enum ProjectCommand {
    /// Load a [`Project`](crate::project::resources::Project) from a path.
    Load(PathBuf),

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
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
