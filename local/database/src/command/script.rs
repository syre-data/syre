//! Script related commands.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::project::{ExcelTemplate, Script as CoreScript};
use syre_core::types::ResourceId;

/// Script related commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum ScriptCommand {
    /// Loads a `Project`'s `Scipt`s.
    ///
    /// # Fields
    /// 1. `Project`'s `ResourceId`.
    LoadProject(ResourceId),

    /// Returns a `ScriptKind`.
    Get(ResourceId),

    /// Updates a `Script`.
    UpdateScript(CoreScript),

    /// Adds a `Script` to a `Project`.
    ///
    /// # Fields
    /// 1. `Project`'s id.
    /// 2. `Script`'s path.
    Add(ResourceId, PathBuf),

    AddExcelTemplate {
        project: ResourceId,
        template: ExcelTemplate,
    },

    /// Removes a script from a `Project`.
    ///
    /// # Fields
    /// 1. `Project`'s id.
    /// 2. `Script`'s id.
    Remove(ResourceId, ResourceId),

    /// Gets the `Project` of a script.
    GetProject(ResourceId),
}
