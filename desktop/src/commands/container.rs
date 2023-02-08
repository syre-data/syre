//! Resources for [`container commands`](thot_desktop_tauri::commands::container).
use serde::Serialize;
use std::path::PathBuf;
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container as CoreContainer, StandardProperties};
use thot_core::types::ResourceId;
use thot_local_database::command::container::AddAssetInfo;

/// Arguments for
/// [`load_container_tree`](thot_desktop_tauri::commands::container::load_container_tree).
#[derive(Serialize)]
pub struct LoadContainerTreeArgs {
    /// Root of the container tree.
    pub root: PathBuf,
}

/// Arguments for commands requiring a [`Container`](CoreContainer) named `container` only.
#[derive(Serialize)]
pub struct ContainerArgs {
    /// [`Container`](CoreContainer) to update.
    pub container: CoreContainer,
}

/// Arguments for [`new_child`](thot_desktop_tauri::commands::container::new_child).
#[derive(Serialize)]
pub struct NewChildArgs {
    /// Name of the child.
    pub name: String,

    /// [`ResourceId`] of the parent [`Container`](thot_core::project::Container).
    pub parent: ResourceId,
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: StandardProperties,
}

/// Arguments to update a [`Container`](thot_core::project::Container)'s
/// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
#[derive(Clone, Serialize)]
pub struct UpdateScriptAssociationsArgs {
    /// [`ResourceId`] of the [`Container`](thot_core::project::Container).
    pub rid: ResourceId,

    // @todo: Issue with deserializing `HashMap` in Tauri, send as string.
    // See: https://github.com/tauri-apps/tauri/issues/6078
    /// Updated script associations.
    pub associations: ScriptMap,
}

/// TEMPORARY
///
/// Intermediate value for [`UpdateScriptAssociationsArgs`] while dealing with
/// (https://github.com/tauri-apps/tauri/issues/6078)
#[derive(Clone, Serialize)]
pub struct UpdateScriptAssociationsStringArgs {
    /// [`ResourceId`] of the [`Container`](thot_core::project::Container).
    pub rid: ResourceId,

    // @todo: Issue with deserializing `HashMap` in Tauri, send as string.
    // See: https://github.com/tauri-apps/tauri/issues/6078
    /// Updated script associations.
    pub associations: String,
}

/// Arguments for [`add_assets`](thot_desktop_tauri::commands::container::add_assets).
#[derive(Serialize, Debug)]
pub struct AddAssetsArgs {
    /// [`ResourceId`] of the [`Container`](CoreContainer).
    pub container: ResourceId,

    /// [`Asset`](thot_core::project::Asset)s to add.
    pub assets: Vec<AddAssetInfo>,
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
