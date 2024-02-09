//! Resources for [`container commands`](syre_desktop_tauri::commands::container).
use super::common::ResourceIdArgs;
use crate::common::{invoke, invoke_result};
use serde::Serialize;
use std::path::PathBuf;
use syre_core::project::container::ScriptMap;
use syre_core::project::{Container, ContainerProperties};
use syre_core::types::ResourceId;
use syre_desktop_lib::types::AddAssetInfo;
use syre_local_database::command::container::{
    BulkUpdateScriptAssociationsArgs, PropertiesUpdate, ScriptAssociationBulkUpdate,
};
use syre_local_database::Result as DbResult;

pub async fn get_container(container: ResourceId) -> Option<Container> {
    invoke("get_container", ResourceIdArgs { rid: container }).await
}

pub async fn get_container_path(container: ResourceId) -> Option<PathBuf> {
    invoke("get_container_path", ResourceIdArgs { rid: container }).await
}

pub async fn update_properties(rid: ResourceId, properties: ContainerProperties) -> DbResult {
    // TODO Issue with serializing `HashMap` of `metadata`. perform manually.
    // See https://github.com/tauri-apps/tauri/issues/6078
    let properties_str = serde_json::to_string(&properties).unwrap();
    let update = UpdatePropertiesStringArgs {
        rid: rid.clone(),
        properties: properties_str,
    };

    invoke_result("update_container_properties", update).await
}

pub async fn bulk_update_properties(
    containers: Vec<ResourceId>,
    update: impl Into<PropertiesUpdate>,
) -> Result<(), String> {
    invoke_result(
        "bulk_update_container_properties",
        BulkUpdatePropertiesArgs {
            rids: containers,
            update: update.into(),
        },
    )
    .await
}

pub async fn update_script_associations(
    container: ResourceId,
    associations: ScriptMap,
) -> DbResult {
    tracing::debug!(?associations);
    // TODO Issue with deserializing `HashMap` in Tauri, send as string.
    // See https://github.com/tauri-apps/tauri/issues/6078
    let associations_str = serde_json::to_string(&associations).unwrap();
    let update = UpdateScriptAssociationsStringArgs {
        rid: container,
        associations: associations_str,
    };

    invoke_result("update_container_script_associations", update).await
}

pub async fn bulk_update_script_associations(
    containers: Vec<ResourceId>,
    update: ScriptAssociationBulkUpdate,
) -> DbResult {
    invoke_result(
        "bulk_update_container_script_associations",
        BulkUpdateScriptAssociationsArgs { containers, update },
    )
    .await
}

pub async fn add_assets_from_info(
    container: ResourceId,
    assets: Vec<AddAssetInfo>,
) -> Result<(), String> {
    invoke_result(
        "add_assets_from_info",
        AddAssetsInfoArgs { container, assets },
    )
    .await
}

pub async fn add_asset_from_contents(
    container: ResourceId,
    name: String,
    contents: Vec<u8>,
) -> Result<(), String> {
    invoke_result(
        "add_asset_from_contents",
        AddAssetContentsArgs {
            container,
            name,
            contents,
        },
    )
    .await
}

/// Arguments for
/// [`load_container_tree`](syre_desktop_tauri::commands::container::load_container_tree).
#[derive(Serialize)]
pub struct LoadContainerTreeArgs {
    /// Root of the container tree.
    pub root: PathBuf,
}

/// Arguments for commands requiring a [`Container`](Container) named `container` only.
#[derive(Serialize)]
pub struct ContainerArgs {
    /// [`Container`](Container) to update.
    pub container: Container,
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: ContainerProperties, // TODO: Issue with serializing `HashMap` of `metadata`. perform manually.
                                         // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesStringArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: String, // TODO: Issue with serializing `HashMap` of `metadata`. perform manually.
                            // Unify with `UpdatePropertiesArgs` once resolved.
                            // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Arguments to update a [`Container`](syre_core::project::Container)'s
/// [`ScriptAssociation`](syre_core::project::ScriptAssociation)s.
#[derive(Clone, Serialize)]
pub struct UpdateScriptAssociationsArgs {
    /// [`ResourceId`] of the [`Container`](syre_core::project::Container).
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
    /// [`ResourceId`] of the [`Container`](syre_core::project::Container).
    pub rid: ResourceId,

    // @todo: Issue with deserializing `HashMap` in Tauri, send as string.
    // Unify with `UpdateScriptAssociationsArgs` once resolved.
    // See: https://github.com/tauri-apps/tauri/issues/6078
    /// Updated script associations.
    pub associations: String,
}

/// Arguments for [`add_assets`](syre_desktop_tauri::commands::container::add_assets).
#[derive(Serialize, Debug)]
pub struct AddAssetsInfoArgs {
    /// [`ResourceId`] of the [`Container`](Container).
    pub container: ResourceId,

    /// [`Asset`](syre_core::project::Asset)s to add.
    pub assets: Vec<AddAssetInfo>,
}

/// Arguments for [`add_asset_windows`](syre_desktop_tauri::commands::container::add_asset_windows).
#[derive(Serialize)]
pub struct AddAssetContentsArgs {
    /// [`ResourceId`] of the [`Container`](Container).
    pub container: ResourceId,

    /// Name of the file.
    pub name: String,

    /// File contents.
    pub contents: Vec<u8>,
}

/// Bulk update resources.
#[derive(Clone, Serialize)]
pub struct BulkUpdatePropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: PropertiesUpdate,
}
