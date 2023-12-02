//! Resources for common commands.
use super::types::ResourcePropertiesUpdate;
use serde::Serialize;
use std::path::PathBuf;
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

/// Bulk update resources.
#[derive(Clone, Serialize)]
pub struct BulkUpdateResourcePropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: ResourcePropertiesUpdate,
}
