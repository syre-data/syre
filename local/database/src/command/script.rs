//! Script related commands.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_core::project::Script as CoreScript;
use thot_core::types::ResourceId;

/// Script related commands.
#[derive(Serialize, Deserialize)]
pub enum ScriptCommand {
    /// Loads a `Project`'s `Scipt`s.
    ///
    /// # Fields
    /// 1. `Project`'s `ResourceId`.
    LoadProject(ResourceId),

    /// Gets a `Script`.
    ///
    /// # Fields
    /// 1. `Script`'s `ResourceId`.
    Get(ResourceId),

    /// Updates a `Script`.
    Update(CoreScript),

    /// Adds a `Script` to a `Project`.
    ///
    /// # Fields
    /// 1. `Project`'s id.
    /// 2. `Script`'s path.
    Add(ResourceId, PathBuf),
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
