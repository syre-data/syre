//! Commands related to containers.
use crate::error::Result;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container, ContainerProperties};
use thot_core::types::ResourceId;
use thot_desktop_lib::error::{Error as LibError, Result as LibResult};
use thot_desktop_lib::types::AddAssetInfo;
use thot_local::common::unique_file_name;
use thot_local::types::AssetFileAction;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::container::{
    BulkUpdateContainerPropertiesArgs, ContainerPropertiesUpdate,
};
use thot_local_database::command::container::{
    BulkUpdateScriptAssociationsArgs, ScriptAssociationBulkUpdate, UpdatePropertiesArgs,
    UpdateScriptAssociationsArgs,
};
use thot_local_database::command::ContainerCommand;
use thot_local_database::Result as DbResult;

/// Retrieves a [`Container`](Container), or `None` if it is not loaded.
#[tauri::command]
pub fn get_container(db: State<DbClient>, rid: ResourceId) -> Option<Container> {
    let container = db
        .send(ContainerCommand::Get(rid).into())
        .expect("could not retrieve `Container`");

    serde_json::from_value(container)
        .expect("could not convert `GetContainer` result to `Container`")
}

/// Updates an existing [`Container`](LocalContainer)'s properties and persists changes to disk.
#[tauri::command]
pub fn update_container_properties(
    db: State<DbClient>,
    rid: ResourceId,
    properties: String, // TODO Issue with deserializing `HashMap` of `metadata`. perform manually.
                        // See: https://github.com/tauri-apps/tauri/issues/6078
                        // properties: ContainerProperties,
) -> LibResult {
    let properties: ContainerProperties = serde_json::from_str(&properties)
        .expect("could not deserialize into `ContainerProperties`");

    let res = db
        .send(ContainerCommand::UpdateProperties(UpdatePropertiesArgs { rid, properties }).into())
        .expect("could not update `Container` properties");

    let res: DbResult = serde_json::from_value(res)
        .expect("could not convert result of `UpdateContainerProperties` from JsValue");

    res.map_err(|err| LibError::Database(format!("{err:?}")))
}

/// Updates an existing [`Container`](LocalContainer)'s script associations and persists changes to disk.
#[tauri::command]
pub fn update_container_script_associations(
    db: State<DbClient>,
    rid: ResourceId,
    associations: String, // TODO Issue with deserializing `HashMap`. perform manually.
                          // See: https://github.com/tauri-apps/tauri/issues/6078
                          // associations: ScriptMap,
) -> Result {
    // TODO Issue with deserializing `HashMap`. perform manually.
    // See: https://github.com/tauri-apps/tauri/issues/6078
    let associations: ScriptMap =
        serde_json::from_str(&associations).expect("could not deserialize into `ScriptMap`");

    let res = db
        .send(
            ContainerCommand::UpdateScriptAssociations(UpdateScriptAssociationsArgs {
                rid,
                associations,
            })
            .into(),
        )
        .expect("could not update `Script` associations");

    let res: DbResult = serde_json::from_value(res)
        .expect("could not convert result of `UpdateContainerScriptAssociations` from JsValue");

    Ok(res?)
}

/// Gets the current location of a [`Container`](LocalContainer).
#[tauri::command]
pub fn get_container_path(db: State<DbClient>, rid: ResourceId) -> Option<PathBuf> {
    let path = db
        .send(ContainerCommand::Path(rid).into())
        .expect("could not get `Container` path");

    let path: Option<PathBuf> = serde_json::from_value(path)
        .expect("could not convert `GetContainerPath` result to `PathBuf`");

    Some(path?)
}

/// Adds [`Asset`](thot_::project::Asset)s to a [`Container`](thot_::project::Container).
#[tauri::command]
pub fn add_assets(db: State<DbClient>, container: ResourceId, assets: Vec<AddAssetInfo>) -> Result {
    let container_path = db.send(ContainerCommand::Path(container).into()).unwrap();
    let Some(container_path) = serde_json::from_value::<Option<PathBuf>>(container_path).unwrap()
    else {
        panic!("could not get container path");
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
            asset_path.push(bucket);
            fs::create_dir_all(asset_path)?; // will trigger folder to be created as container by database.
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
pub fn add_asset_windows(
    db: State<DbClient>,
    container: ResourceId,
    name: String,
    contents: Vec<u8>,
) -> Result {
    // create file
    let path = db
        .send(ContainerCommand::Path(container.clone()).into())
        .expect("could not get `Container` path");

    let path: Option<PathBuf> =
        serde_json::from_value(path).expect("could not convert result of `GetPath` to `PathBuf`");

    let mut path = path.expect("could not get `Container` path");
    path.push(name);
    let path = unique_file_name(path).expect("could not create a unique file name");
    fs::write(&path, contents).expect("could not write to file");
    Ok(())
}

#[tauri::command]
pub fn bulk_update_container_properties(
    db: State<DbClient>,
    rids: Vec<ResourceId>,
    update: ContainerPropertiesUpdate,
) -> Result {
    let res = db.send(
        ContainerCommand::BulkUpdateProperties(BulkUpdateContainerPropertiesArgs { rids, update })
            .into(),
    );

    // TODO Handle errors.
    res.expect("could not update Containers");
    Ok(())
}

#[tauri::command]
pub fn bulk_update_container_script_associations(
    db: State<DbClient>,
    containers: Vec<ResourceId>,
    update: ScriptAssociationBulkUpdate,
) -> Result {
    // TODO Handle errors.
    db.send(
        ContainerCommand::BulkUpdateScriptAssociations(BulkUpdateScriptAssociationsArgs {
            containers,
            update,
        })
        .into(),
    )
    .unwrap();
    Ok(())
}
