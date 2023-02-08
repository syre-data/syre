//! [`Asset`](CoreAsset) functionality.
use crate::error::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use thot_core::project::StandardProperties;
use thot_core::types::ResourceId;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::AssetCommand;

/// Gets [`Asset`](CoreAsset)s.
///
/// # Returns
/// `Vec<[Asset](CoreAsset)>` where [`Asset`](CoreAsset)s that
/// are not found are ignored.
#[tauri::command]
pub fn get_assets(
    db: State<DbClient>,
    assets: Vec<ResourceId>,
) -> HashMap<ResourceId, Option<PathBuf>> {
    let assets = db.send(AssetCommand::GetAssets(assets).into());
    serde_json::from_value(assets).expect("could not convert result of `GetAssets` to `Vec<Asset>`")
}

/// Update an [`Asset`](CoreAsset).
#[tauri::command]
pub fn update_asset_properties(
    db: State<DbClient>,
    rid: ResourceId,
    properties: StandardProperties,
) -> Result {
    db.send(AssetCommand::UpdateAssetProperties(rid, properties).into());
    Ok(())
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
