//! `Asset` functionality.
use crate::error::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::result::Result as StdResult;
use tauri::State;
use thot_core::project::{Asset, AssetProperties};
use thot_core::types::ResourceId;
use thot_desktop_lib::error::{RemoveAsset, Trash as TrashError};
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
pub fn remove_asset(db: State<DbClient>, rid: ResourceId) -> StdResult<(), RemoveAsset> {
    let path = match db.send(AssetCommand::Remove(rid).into()) {
        Ok(res) => match serde_json::from_value::<DbResult<Option<(Asset, PathBuf)>>>(res).unwrap()
        {
            Ok(Some((_asset, path))) => path,
            Ok(None) => return Err(RemoveAsset::Database("asset does not exist".to_string())),
            Err(err) => return Err(RemoveAsset::Database(format!("{err:?}"))),
        },

        Err(err) => return Err(RemoveAsset::ZMQ(format!("{err:?}"))),
    };

    match trash::delete(path) {
        Ok(_) => Ok(()),

        Err(trash::Error::CanonicalizePath { original: _ }) => Err(TrashError::NotFound.into()),

        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_os = "ios"),
            not(target_os = "android")
        ))]
        Err(trash::Error::FileSystem { path, source })
            if source.kind() == io::ErrorKind::NotFound =>
        {
            Err(TrashError::NotFound.into())
        }

        Err(trash::Error::Unknown { description }) => {
            if cfg!(target_os = "windows") {
                let err = handle_trash_error_unknown_windows(description);
                Err(err.into())
            } else if cfg!(target_os = "macos") {
                let err = handle_trash_error_unknown_macos(description);
                Err(err.into())
            } else {
                Err(TrashError::Other(description).into())
            }
        }

        Err(err) => Err(TrashError::Other(format!("{err:?}")).into()),
    }
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

fn handle_trash_error_unknown_windows(description: String) -> TrashError {
    // all windows os errors are mapped to `Unknown`.
    // Can parse string for error code to map.
    // See https://github.com/Byron/trash-rs/issues/96.
    // See https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-
    let re = regex::Regex::new(r"os error (\d+)").unwrap();
    match re.captures(&description) {
        None => TrashError::Other(description),
        Some(captures) => {
            let code: i32 = captures.get(1).unwrap().as_str().parse().unwrap();
            match code {
                2 | 3 => TrashError::NotFound,
                _ => TrashError::Other(description),
            }
        }
    }
}

fn handle_trash_error_unknown_macos(description: String) -> TrashError {
    let re = regex::Regex::new(r"\((-?\d+)\)\s*$").unwrap();
    match re.captures(&description) {
        None => TrashError::Other(description),
        Some(captures) => {
            let code: i32 = captures.get(1).unwrap().as_str().parse().unwrap();
            match code {
                -10010 => TrashError::NotFound,
                _ => TrashError::Other(description),
            }
        }
    }
}
