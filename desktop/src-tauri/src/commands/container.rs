//! Commands related to containers.
use crate::error::Result;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container as CoreContainer, StandardProperties};
use thot_core::types::{Creator, ResourceId, UserId};
use thot_local::project::container;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::container::{
    AddAssetInfo, AddAssetsArgs, NewChildArgs, UpdatePropertiesArgs, UpdateScriptAssociationsArgs,
};
use thot_local_database::command::ContainerCommand;
use thot_local_database::Result as DbResult;

/// Loads a [`Container`]Tree from path.
/// Adds containers into the [`ContainerStore`].
#[tauri::command]
pub fn load_container_tree(db: State<DbClient>, root: PathBuf) -> Result<CoreContainer> {
    let root = db.send(ContainerCommand::LoadTree(root).into());
    let root: DbResult<CoreContainer> = serde_json::from_value(root)
        .expect("could not convert `LoadContainerTree` result to a `Container`");

    Ok(root?)
}

/// Initializes a directory as a [`Container`](LocalContainer).
///
/// # Argument
/// 1. `path`: Path to the desired child directory.
/// 2. `container`: [`Container`](CoreContainer) to initialize with.
///     The [`ResourceId`] is ignored.
///
/// # Returns
/// [`ResourceId`] of the initialized [`Container`](CoreContainer).
///
/// # See also
/// + [`thot_local::project::container::init`] for details.
#[tauri::command]
pub fn init_container(
    db: State<DbClient>,
    app_state: State<AppState>,
    path: PathBuf,
) -> Result<ResourceId> {
    // create container
    let mut container = CoreContainer::default();
    let user = app_state
        .user
        .lock()
        .expect("could not lock `AppState.user`")
        .clone();

    let user = user.map(|user| UserId::Id(user.rid));
    container.properties.creator = Creator::User(user);

    let _rid = container::init_from(&path, container)?;

    // load and store container
    let container = db.send(ContainerCommand::Load(path).into());
    let container: DbResult<CoreContainer> = serde_json::from_value(container)
        .expect("could not convert `LoadContainer` result to a `Container`");

    Ok(container?.rid)
}

/// Creates a new child [`Container`](LocalContainer).
/// Adds the child into the [`ContainerStore`].
///
/// # Arguments
/// 1. `name`: Name of the child.
/// 2. `parent`: [`ResourceId`] of the parent [`Container`](LocalContainer).
#[tauri::command]
pub fn new_child(db: State<DbClient>, name: String, parent: ResourceId) -> Result<CoreContainer> {
    let child = db.send(ContainerCommand::NewChild(NewChildArgs { name, parent }).into());
    let child: DbResult<CoreContainer> = serde_json::from_value(child)
        .expect("could not convert `NewChild` result to a `Container`");

    Ok(child?)
}

/// Retrieves a [`Container`](CoreContainer), or `None` if it is not loaded.
#[tauri::command]
pub fn get_container(db: State<DbClient>, rid: ResourceId) -> Option<CoreContainer> {
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
) -> Result {
    let properties: StandardProperties =
        serde_json::from_str(&properties).expect("could not deserialize into `StandardProperties`");

    let res = db
        .send(ContainerCommand::UpdateProperties(UpdatePropertiesArgs { rid, properties }).into());

    let res: DbResult = serde_json::from_value(res)
        .expect("could not convert result of `UpdateContainerProperties` from JsValue");

    Ok(res?)
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

/// Adds [`Asset`](thot_core::project::Asset)s to a [`Container`](thot_core::project::Container).
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

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
