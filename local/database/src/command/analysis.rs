//! Analysis related commands.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::project::{ExcelTemplate, Script};
use syre_core::types::ResourceId;

/// Analysis related commands.
#[derive(Serialize, Deserialize, Debug)]
pub enum AnalysisCommand {
    /// Loads a `Project`'s analyses.
    ///
    /// # Fields
    /// 1. `Project`'s `ResourceId`.
    LoadProject(ResourceId),

    /// Get an analysis by id.
    Get(ResourceId),

    /// Updates a [`Script`].
    UpdateScript(Script),

    /// Adds a `Script` to a `Project` by path.
    ///
    /// # Fields
    /// 1. `Project`'s id.
    /// 2. `Script`'s path.
    AddScript(ResourceId, PathBuf),

    AddExcelTemplate {
        project: ResourceId,
        template: ExcelTemplate,
    },

    /// Removes an analysis from a `Project`.
    Remove {
        project: ResourceId,
        script: ResourceId,
    },

    /// Gets the `Project` of an analysis.
    GetProject(ResourceId),
}
