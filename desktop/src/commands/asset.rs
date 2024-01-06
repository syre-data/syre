//! Resources for [`Asset`](CoreAsset) functionality.
use super::common::ResourceIdArgs;
use crate::common::invoke_result;
use serde::Serialize;
use thot_core::project::{Asset, AssetProperties};
use thot_core::types::ResourceId;
use thot_local_database::command::asset::{BulkUpdatePropertiesArgs, PropertiesUpdate};
use thot_local_database::Result as DbResult;

pub async fn update_properties(asset: ResourceId, properties: AssetProperties) -> DbResult {
    invoke_result(
        "update_asset_properties",
        &UpdatePropertiesStringArgs {
            rid: asset,
            properties: serde_json::to_string(&properties).unwrap(),
        },
    )
    .await
}

pub async fn bulk_update_properties(
    assets: Vec<ResourceId>,
    update: impl Into<PropertiesUpdate>,
) -> Result<(), String> {
    invoke_result(
        "bulk_update_asset_properties",
        BulkUpdatePropertiesArgs {
            rids: assets,
            update: update.into(),
        },
    )
    .await
}

pub async fn remove_asset(asset: ResourceId) -> Result<(), String> {
    invoke_result("remove_asset", ResourceIdArgs { rid: asset }).await
}

#[derive(Serialize)]
pub struct AssetArgs {
    pub asset: Asset,
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: AssetProperties, // TODO: Issue with serializing `HashMap` of `metadata`. perform manually.
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
