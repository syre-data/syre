//! Container related commands.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_core::db::StandardSearchFilter;
use thot_core::project::container::ScriptMap;
use thot_core::project::StandardProperties;
use thot_core::types::ResourceId;
use thot_local::types::AssetFileAction;

/// Container related commands.
#[derive(Serialize, Deserialize)]
pub enum ContainerCommand {
    /// Retrieves a [`Container`](CoreContainer) by [`ResourceId`].
    Get(ResourceId),

    /// Retrieves a [`Container`](CoreContainer) by its path.
    ByPath(PathBuf),

    /// Retrieves [`Container`](CoreContainer)s based on a filter.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    Find(ResourceId, StandardSearchFilter),

    /// Retrieves [`Container`](CoreContainer)s based on a filter.
    /// Lineage is compiled.
    ///
    /// # Fields
    /// 1. Root `Container`.
    /// 2. Search filter.
    FindWithMetadata(ResourceId, StandardSearchFilter),

    /// Updates a [`Container`](CoreContainer)'s properties.
    UpdateProperties(UpdatePropertiesArgs),

    /// Updates a [`Container`](CoreContainer)'s
    /// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
    UpdateScriptAssociations(UpdateScriptAssociationsArgs),

    /// Add [`Asset`](CoreAsset)s to a [`Container`](CoreContainer).
    ///
    /// # Notes
    /// + If an [`Asset`] with a given path already exists, the file name is
    /// changed to be unique.
    AddAssets(AddAssetsArgs),

    /// Gets the path of a [`Container`](thot_local::project::resources::Container).
    GetPath(ResourceId),

    /// Gets the parent of a [`Container`](thot_core::project::Container).
    Parent(ResourceId),
}

// *****************
// *** Arguments ***
// *****************

/// Arguments for updating a resource's [`StandardProperties`].
#[derive(Serialize, Deserialize)]
pub struct UpdatePropertiesArgs {
    pub rid: ResourceId,
    pub properties: StandardProperties,
}

/// Arguments for updating a [`Container`](CoreContainer)'s
/// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
#[derive(Serialize, Deserialize)]
pub struct UpdateScriptAssociationsArgs {
    pub rid: ResourceId,
    pub associations: ScriptMap,
}

/// Arguments for adding [`Asset`](CoreAsset)s to a [`Container`](CoreContainer).
#[derive(Serialize, Deserialize)]
pub struct AddAssetsArgs {
    pub container: ResourceId,
    pub assets: Vec<AddAssetInfo>,
}

// **********************
// *** Add Asset Info ***
// **********************

// @todo: Merge with `thot_local::types::AssetFileAction`.
/// Info for adding an [`Asset`](thot_core::project::Asset).
#[derive(Serialize, Deserialize, Debug)]
pub struct AddAssetInfo {
    /// Path of the file to make an [`Asset`](thot_core::project::Asset).
    pub path: PathBuf,

    /// How to handle the file on disk.
    pub action: AssetFileAction,

    /// The bucket to place the [`Asset`](thot_core::project::Asset)'s file in.
    pub bucket: Option<PathBuf>,
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
