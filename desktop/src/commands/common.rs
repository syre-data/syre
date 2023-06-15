//! Resources for common commands.
use super::types::StandardPropertiesUpdate;
use serde::Serialize;
use std::path::PathBuf;
use thot_core::project::StandardProperties;
use thot_core::types::ResourceId;

/// Used for functions that do not accept arguments.
#[derive(Serialize)]
pub struct EmptyArgs {}

/// Used for functions that require a [`ResourceId`] named `rid` as its only argument.
#[derive(Serialize)]
pub struct ResourceIdArgs {
    pub rid: ResourceId,
}

/// Used for functions that require a [`PathBuf`] named `path` as its only argument.
#[derive(Serialize)]
pub struct PathBufArgs {
    /// Path to the project root.
    pub path: PathBuf,
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: StandardProperties, // @todo: Issue with serializing `HashMap` of `metadata`. perform manually.
                                        // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesStringArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: String, // @todo: Issue with serializing `HashMap` of `metadata`. perform manually.
                            // Unify with `UpdatePropertiesArgs` once resolved.
                            // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Bulk update resources.
#[derive(Clone, Serialize)]
pub struct BulkUpdatePropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: StandardPropertiesUpdate,
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
