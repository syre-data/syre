//! Resources for [`Asset`](CoreAsset) functionality.
use super::common::ResourceIdArgs;
use crate::common::invoke_result;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::project::{Asset, AssetProperties};
use syre_core::types::ResourceId;
use syre_desktop_lib::error::RemoveResource;
use syre_local_database::command::asset::{BulkUpdatePropertiesArgs, PropertiesUpdate};
use syre_local_database::Result as DbResult;

pub async fn update_properties(asset: ResourceId, properties: AssetProperties) -> DbResult {
    invoke_result(
        "update_asset_properties",
        &UpdatePropertiesArgs {
            rid: asset,
            properties,
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

pub async fn remove_asset(asset: ResourceId) -> Result<Option<(Asset, PathBuf)>, RemoveResource> {
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
    pub properties: AssetProperties,
}
