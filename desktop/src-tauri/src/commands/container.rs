//! Commands related to containers.
use crate::error::Result;
use std::fs;
use std::path::PathBuf;
use std::result::Result as StdResult;
use syre_core::project::container::AnalysisMap;
use syre_core::project::{Container, ContainerProperties};
use syre_core::types::ResourceId;
use syre_desktop_lib::types::AddAssetInfo;
use syre_local::common::unique_file_name;
use syre_local::types::AssetFileAction;
use syre_local_database::client::Client as DbClient;
use syre_local_database::command::container::{AnalysisAssociationBulkUpdate, PropertiesUpdate};
use syre_local_database::error::server::{
    Update as UpdateError, UpdateContainer as UpdateContainerError,
};
use syre_local_database::Result as DbResult;
use tauri::State;

/// Retrieves a [`Container`](Container), or `None` if it is not loaded.
#[tauri::command]
pub fn get_container(db: State<DbClient>, rid: ResourceId) -> Option<Container> {
    db.container().get(rid).unwrap()
}

/// Updates an existing [`Container`](LocalContainer)'s properties and persists changes to disk.
#[tauri::command]
pub fn update_container_properties(
    db: State<DbClient>,
    rid: ResourceId,
    properties: String, // TODO Issue with deserializing enum with Option. perform manually.
                        // See: https://github.com/tauri-apps/tauri/issues/5993
                        // properties: ContainerProperties,
) -> StdResult<(), UpdateContainerError> {
    let properties: ContainerProperties = serde_json::from_str(&properties).unwrap();
    db.container().update_properties(rid, properties).unwrap()
}

/// Updates an existing [`Container`](LocalContainer)'s script associations and persists changes to disk.
#[tauri::command]
pub fn update_container_analysis_associations(
    db: State<DbClient>,
    rid: ResourceId,
    associations: AnalysisMap,
) -> StdResult<(), UpdateError> {
    db.container()
        .update_analysis_associations(rid, associations)
        .unwrap()
}

/// Gets the current location of a [`Container`](LocalContainer).
#[tauri::command]
pub fn get_container_path(db: State<DbClient>, rid: ResourceId) -> Option<PathBuf> {
    db.container().path(rid).unwrap()
}

/// Adds [`Asset`](syre_core::project::Asset)s to a [`Container`].
#[tauri::command]
pub fn add_assets_from_info(
    db: State<DbClient>,
    container: ResourceId,
    assets: Vec<AddAssetInfo>,
) -> Result {
    let Some(container_path) = db.container().path(container).unwrap() else {
        panic!("could not find container path");
    };

    for AddAssetInfo {
        path,
        action,
        bucket,
    } in assets
    {
        let mut asset_path = container_path.clone();
        if let Some(bucket) = bucket {
            todo!();
            // asset_path.push(bucket);
            // fs::create_dir_all(asset_path)?; // will trigger folder to be created as container by database.
        }
        asset_path.push(path.file_name().unwrap());

        match action {
            AssetFileAction::Copy => {
                fs::copy(path, asset_path)?;
            }
            AssetFileAction::Move => todo!(),
            AssetFileAction::Reference => todo!(),
        }
    }

    Ok(())
}

#[tauri::command]
pub fn add_asset_from_contents(
    db: State<DbClient>,
    container: ResourceId,
    name: String,
    contents: Vec<u8>,
) -> Result {
    // create file
    let Some(mut path) = db.container().path(container).unwrap() else {
        panic!("could not get container path");
    };

    path.push(name);
    let path = unique_file_name(path).unwrap();
    fs::write(&path, contents).unwrap();
    Ok(())
}

#[tauri::command]
pub fn bulk_update_container_properties(
    db: State<DbClient>,
    rids: Vec<ResourceId>,
    update: PropertiesUpdate,
) -> DbResult {
    db.container().bulk_update_properties(rids, update).unwrap()
}

#[tauri::command]
pub fn bulk_update_container_analysis_associations(
    db: State<DbClient>,
    containers: Vec<ResourceId>,
    update: AnalysisAssociationBulkUpdate,
) -> DbResult {
    db.container()
        .bulk_update_analysis_associations(containers, update)
        .unwrap()
}
