//! Commands related to containers.
use crate::error::Result;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container, StandardProperties};
use thot_core::types::ResourceId;
use thot_desktop_lib::error::{Error as LibError, Result as LibResult};
use thot_local::common::unique_file_name;
use thot_local::types::AssetFileAction;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::container::{
    AddAssetInfo, AddAssetsArgs, UpdatePropertiesArgs, UpdateScriptAssociationsArgs,
};
use thot_local_database::command::ContainerCommand;
use thot_local_database::Result as DbResult;

/// Retrieves a [`Container`](Container), or `None` if it is not loaded.
#[tauri::command]
pub fn get_container(db: State<DbClient>, rid: ResourceId) -> Option<Container> {
    let container = db.send(ContainerCommand::Get(rid).into());
    serde_json::from_value(container)
        .expect("could not convert `GetContainer` result to `Container`")
}

/// Updates an existing [`Container`](LocalContainer)'s properties and persists changes to disk.
#[tauri::command]
pub fn update_container_properties(
    db: State<DbClient>,
    rid: ResourceId,
    properties: String, // @todo: Issue with deserializing `HashMap` of `metadata`. perform manually.
                        // See: https://github.com/tauri-apps/tauri/issues/6078
                        // properties: StandardProperties,
) -> LibResult {
    let properties: StandardProperties =
        serde_json::from_str(&properties).expect("could not deserialize into `StandardProperties`");

    let res = db
        .send(ContainerCommand::UpdateProperties(UpdatePropertiesArgs { rid, properties }).into());

    let res: DbResult = serde_json::from_value(res)
        .expect("could not convert result of `UpdateContainerProperties` from JsValue");

    res.map_err(|err| LibError::Database(format!("{err:?}")))
}

/// Updates an existing [`Container`](LocalContainer)'s script associations and persists changes to disk.
#[tauri::command]
pub fn update_container_script_associations(
    db: State<DbClient>,
    rid: ResourceId,
    associations: String, // @todo: Issue with deserializing `HashMap`. perform manually.
                          // See: https://github.com/tauri-apps/tauri/issues/6078
                          // associations: ScriptMap,
) -> Result {
    // @todo: Issue with deserializing `HashMap`. perform manually.
    // See: https://github.com/tauri-apps/tauri/issues/6078
    let associations: ScriptMap =
        serde_json::from_str(&associations).expect("could not deserialize into `ScriptMap`");

    let res = db.send(
        ContainerCommand::UpdateScriptAssociations(UpdateScriptAssociationsArgs {
            rid,
            associations,
        })
        .into(),
    );

    let res: DbResult = serde_json::from_value(res)
        .expect("could not convert result of `UpdateContainerScriptAssociations` from JsValue");

    Ok(res?)
}

/// Gets the current location of a [`Container`](LocalContainer).
#[tauri::command]
pub fn get_container_path(db: State<DbClient>, rid: ResourceId) -> Result<Option<PathBuf>> {
    let path = db.send(ContainerCommand::GetPath(rid).into());
    let path: DbResult<Option<PathBuf>> = serde_json::from_value(path)
        .expect("could not convert `GetContainerPath` result to `PathBuf`");

    Ok(path?)
}

/// Adds [`Asset`](thot_::project::Asset)s to a [`Container`](thot_::project::Container).
#[tauri::command]
pub fn add_assets(
    db: State<DbClient>,
    container: ResourceId,
    assets: Vec<AddAssetInfo>,
) -> Result<Vec<ResourceId>> {
    let asset_rids =
        db.send(ContainerCommand::AddAssets(AddAssetsArgs { container, assets }).into());

    let asset_rids: DbResult<Vec<ResourceId>> = serde_json::from_value(asset_rids)
        .expect("could not convert `AddAssets` result to `Vec<ResourceId>`");

    Ok(asset_rids?)
}

#[tauri::command]
pub fn add_asset_windows(
    db: State<DbClient>,
    container: ResourceId,
    name: String,
    contents: Vec<u8>,
) -> Result<Vec<ResourceId>> {
    // create file
    let path = db.send(ContainerCommand::GetPath(container.clone()).into());
    let path: DbResult<Option<PathBuf>> =
        serde_json::from_value(path).expect("could not convert result of `GetPath` to `PathBuf`");
    let mut path = path
        .expect("could not get `Container` path")
        .expect("`Container` path not found");
    path.push(name);
    let path = unique_file_name(path).expect("could not create a unique file name");

    fs::write(&path, contents).expect("could not write to file");

    // add asset
    let assets = vec![AddAssetInfo {
        path,
        action: AssetFileAction::Move,
        bucket: None,
    }];

    let asset_rids =
        db.send(ContainerCommand::AddAssets(AddAssetsArgs { container, assets }).into());

    let asset_rids: DbResult<Vec<ResourceId>> = serde_json::from_value(asset_rids)
        .expect("could not convert `AddAssets` result to `Vec<ResourceId>`");

    Ok(asset_rids?)
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
