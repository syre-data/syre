//! `Asset` functionality.
use super::utils;
use crate::error::Result;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use syre_core::project::{Asset, AssetProperties};
use syre_core::types::ResourceId;
use syre_desktop_lib::error::{RemoveResource as RemoveResourceError, Trash as TrashError};
use syre_local_database::client::Client as DbClient;
use syre_local_database::command::asset::PropertiesUpdate;
use syre_local_database::error::server::Update as UpdateError;
use tauri::State;

/// Gets `Asset`s.
#[tauri::command]
pub fn get_assets(db: State<DbClient>, assets: Vec<ResourceId>) -> Vec<Asset> {
    db.asset().get_many(assets).unwrap()
}

/// Update an `Asset`'s properties.
#[tauri::command]
pub fn update_asset_properties(
    db: State<DbClient>,
    rid: ResourceId,
    properties: AssetProperties,
) -> StdResult<(), UpdateError> {
    db.asset().update_properties(rid, properties).unwrap()
}

/// Remove an `Asset`.
#[tauri::command]
pub fn remove_asset(db: State<DbClient>, rid: ResourceId) -> StdResult<(), RemoveResourceError> {
    let remove_asset_from_db = |asset: ResourceId| -> StdResult<PathBuf, RemoveResourceError> {
        match db.asset().remove(asset).unwrap() {
            Ok(Some((_asset, path))) => Ok(path),
            Ok(None) => {
                return Err(RemoveResourceError::Database(
                    "asset does not exist".to_string(),
                ))
            }
            Err(err) => return Err(RemoveResourceError::Database(format!("{err:?}"))),
        }
    };

    let Some(path) = db.asset().path(rid.clone()).unwrap() else {
        tracing::debug!("asset {rid:?} path not found");
        return Err(RemoveResourceError::Database(
            "Could not get Asset's path".into(),
        ));
    };

    match trash::delete(path) {
        Ok(_) => Ok(()),

        Err(trash::Error::CanonicalizePath { original: _ }) => match remove_asset_from_db(rid) {
            Ok(_) => Err(TrashError::NotFound.into()),
            Err(err) => Err(err),
        },

        Err(trash::Error::CouldNotAccess { target }) => {
            if Path::new(&target).exists() {
                Err(TrashError::PermissionDenied.into())
            } else {
                match remove_asset_from_db(rid) {
                    Ok(_) => Err(TrashError::NotFound.into()),
                    Err(err) => Err(err),
                }
            }
        }

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

        Err(trash::Error::Os { code, description }) => {
            match utils::trash::convert_os_error(code, description) {
                TrashError::NotFound => {
                    remove_asset_from_db(rid)?;
                    Err(TrashError::NotFound.into())
                }

                err => Err(err.into()),
            }
        }

        Err(trash::Error::Unknown { description }) => Err(TrashError::Other(description).into()),

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
    Ok(db.asset().bulk_update_properties(rids, update).unwrap()?)
}
