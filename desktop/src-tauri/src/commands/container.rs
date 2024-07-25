use std::{fs, path::PathBuf};
use syre_core::{
    project::{AssetProperties, ContainerProperties},
    types::ResourceId,
};
use syre_desktop_lib::command::container::error;
use syre_local as local;
use syre_local_database as db;

/// Rename a container folder.
///
/// # Arguments
/// 1. `project`
/// 2. `container`: Current container path.
/// Path should be absolute from graph root.
/// 3. `name`: New name.
#[tauri::command]
pub fn container_rename(
    db: tauri::State<db::Client>,
    project: ResourceId,
    container: PathBuf,
    name: String, // TODO: Should be an `OsString` but need to specify custom deserializer
                  // `syre_local_database::serde_os_string`.
) -> Result<(), error::Rename> {
    let Some((project_path, project_data)) = db.project().get_by_id(project.clone()).unwrap()
    else {
        return Err(error::Rename::ProjectNotFound);
    };

    let db::state::DataResource::Ok(properties) = project_data.properties() else {
        panic!("invalid state");
    };
    assert_eq!(properties.rid(), &project);

    let data_root = project_path.join(&properties.data_root);
    let path = db::common::container_system_path(data_root, container);
    let mut path_new = path.clone();
    path_new.set_file_name(name);
    if path_new.exists() {
        return Err(error::Rename::NameCollision);
    }

    if let Err(err) = fs::rename(path, path_new) {
        return Err(error::Rename::Rename(err.kind()));
    }

    Ok(())
}

/// Update a container's properties.
#[tauri::command]
pub fn container_properties_update(
    db: tauri::State<db::Client>,
    project: ResourceId,
    container: PathBuf,
    properties: ContainerProperties,
) -> Result<(), error::Update> {
    let Some((project_path, project_data)) = db.project().get_by_id(project.clone()).unwrap()
    else {
        return Err(error::Update::ProjectNotFound);
    };

    let db::state::DataResource::Ok(project_properties) = project_data.properties() else {
        panic!("invalid state");
    };
    assert_eq!(project_properties.rid(), &project);

    let data_root = project_path.join(&project_properties.data_root);
    let path = db::common::container_system_path(data_root, container);
    let mut container = local::loader::container::Loader::load_from_only_properties(&path).unwrap();
    container.properties = properties;
    if let Err(err) = container.save(&path) {
        return Err(error::Update::Save(err.kind()));
    }

    Ok(())
}

/// Update a container's properties.
#[tauri::command]
pub fn asset_properties_update(
    db: tauri::State<db::Client>,
    project: ResourceId,
    container: PathBuf,
    asset: PathBuf,
    // properties: AssetProperties,
    properties: String, // TODO: Issue with serializing enum with Option. perform manually.
                        // See: https://github.com/tauri-apps/tauri/issues/5993
) -> Result<(), error::Update> {
    let properties = serde_json::from_str::<AssetProperties>(&properties).unwrap();
    let Some((project_path, project_data)) = db.project().get_by_id(project.clone()).unwrap()
    else {
        return Err(error::Update::ProjectNotFound);
    };

    let db::state::DataResource::Ok(project_properties) = project_data.properties() else {
        panic!("invalid state");
    };
    assert_eq!(project_properties.rid(), &project);

    let data_root = project_path.join(&project_properties.data_root);
    let path = db::common::container_system_path(data_root, container);
    let mut assets = local::project::resources::Assets::load_from(path)
        .map_err(|err| error::Update::Load(err))?;
    let asset = assets
        .iter_mut()
        .find(|asset_state| asset_state.path == asset)
        .unwrap();
    asset.properties = properties;
    if let Err(err) = assets.save() {
        return Err(error::Update::Save(err.kind()));
    }

    Ok(())
}
