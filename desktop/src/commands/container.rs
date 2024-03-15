//! Resources for [`container commands`](syre_desktop_tauri::commands::container).
use super::common::ResourceIdArgs;
use crate::invoke::{invoke, invoke_result};
use serde::Serialize;
use std::path::PathBuf;
use syre_core::project::container::AnalysisMap;
use syre_core::project::{Container, ContainerProperties};
use syre_core::types::ResourceId;
use syre_desktop_lib::types::AddAssetInfo;
use syre_local_database::command::container::{
    AnalysisAssociationBulkUpdate, BulkUpdateAnalysisAssociationsArgs, PropertiesUpdate,
};
use syre_local_database::Result as DbResult;

pub async fn get_container(container: ResourceId) -> Option<Container> {
    invoke("get_container", ResourceIdArgs { rid: container }).await
}

pub async fn get_container_path(container: ResourceId) -> Option<PathBuf> {
    invoke("get_container_path", ResourceIdArgs { rid: container }).await
}

pub async fn update_properties(rid: ResourceId, properties: ContainerProperties) -> DbResult {
    // TODO Issue with serializing enum with Option. perform manually.
    // See https://github.com/tauri-apps/tauri/issues/5993
    let properties = serde_json::to_string(&properties).unwrap();
    let update = UpdatePropertiesStringArgs {
        rid: rid.clone(),
        properties,
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

pub async fn update_analysis_associations(
    container: ResourceId,
    associations: AnalysisMap,
) -> DbResult {
    let update = UpdateAnalysisAssociationsArgs {
        rid: container,
        associations,
    };

    invoke_result("update_container_analysis_associations", update).await
}

pub async fn bulk_update_analysis_associations(
    containers: Vec<ResourceId>,
    update: AnalysisAssociationBulkUpdate,
) -> DbResult {
    invoke_result(
        "bulk_update_container_script_associations",
        BulkUpdateAnalysisAssociationsArgs { containers, update },
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
#[derive(Clone, Debug, Serialize)]
pub struct UpdatePropertiesArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: ContainerProperties, // TODO: Issue with serializing enum with Option. perform manually.
                                         // See: https://github.com/tauri-apps/tauri/issues/5993
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesStringArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: String, // TODO: Issue with serializing enum with Option. perform manually.
                            // See: https://github.com/tauri-apps/tauri/issues/5993
}

/// Arguments to update a [`Container`](syre_core::project::Container)'s
/// [`AnalysisAssociation`](syre_core::project::AnalysisAssociation)s.
#[derive(Clone, Serialize)]
pub struct UpdateAnalysisAssociationsArgs {
    /// [`ResourceId`] of the [`Container`](syre_core::project::Container).
    pub rid: ResourceId,

    /// Updated script associations.
    pub associations: AnalysisMap,
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
