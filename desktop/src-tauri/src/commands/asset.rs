//! `Asset` functionality.
use crate::error::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use thot_core::project::AssetProperties;
use thot_core::types::ResourceId;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::asset::{BulkUpdatePropertiesArgs, PropertiesUpdate};
use thot_local_database::command::AssetCommand;
use thot_local_database::Result as DbResult;

/// Gets `Asset`s.
#[tauri::command]
pub fn get_assets(
    db: State<DbClient>,
    assets: Vec<ResourceId>,
) -> HashMap<ResourceId, Option<PathBuf>> {
    let assets = db
        .send(AssetCommand::GetMany(assets).into())
        .expect("could not retrieve `Asset`s");

    serde_json::from_value(assets).expect("could not convert result of `GetAssets` to `Vec<Asset>`")
}

/// Update an `Asset`'s properties.
#[tauri::command]
pub fn update_asset_properties(
    db: State<DbClient>,
    rid: ResourceId,
    properties: String,
) -> DbResult {
    let properties: AssetProperties = serde_json::from_str(&properties).unwrap();
    let res = db
        .send(
            AssetCommand::UpdateProperties {
                asset: rid,
                properties,
            }
            .into(),
        )
        .unwrap();

    serde_json::from_value(res).unwrap()
}

/// Remove an `Asset`.
#[tauri::command]
pub fn remove_asset(db: State<DbClient>, rid: ResourceId) -> Result {
    let path = db.send(AssetCommand::Path(rid).into())?;
    let Some(path) = serde_json::from_value::<Option<PathBuf>>(path).unwrap() else {
        panic!("asset not found");
    };

    trash::delete(path).unwrap();
    Ok(())
}

/// Bulk update the porperties of `Asset`s.
#[tauri::command]
pub fn bulk_update_asset_properties(
    db: State<DbClient>,
    rids: Vec<ResourceId>,
    update: PropertiesUpdate,
) -> Result {
    let res = db
        .send(AssetCommand::BulkUpdateProperties(BulkUpdatePropertiesArgs { rids, update }).into());

    // TODO Handle errors.
    res.expect("could not update `Asset`s");
    Ok(())
}
