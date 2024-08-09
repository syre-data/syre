use std::{
    fs,
    path::{Path, PathBuf},
};
use syre_core::{
    project::{AnalysisAssociation, AssetProperties, ContainerProperties},
    types::ResourceId,
};
use syre_desktop_lib::{
    self as lib,
    command::container::{
        bulk::{self, PropertiesUpdate},
        error,
    },
};
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

/// Update a container's analysis associations.
#[tauri::command]
pub fn container_analysis_associations_update(
    db: tauri::State<db::Client>,
    project: ResourceId,
    container: PathBuf,
    associations: Vec<AnalysisAssociation>,
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
    container.analyses = associations;
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

/// Rename multiple container folders.
///
/// # Arguments
/// 1. `project`
/// 2. `containers`: Current container path.
/// Path should be absolute from graph root.
/// 3. `name`: New name.
#[tauri::command]
pub fn container_rename_bulk(
    db: tauri::State<db::Client>,
    project: ResourceId,
    containers: Vec<PathBuf>,
    name: String, // TODO: Should be an `OsString` but need to specify custom deserializer
                  // `syre_local_database::serde_os_string`.
) -> Result<Vec<Result<(), lib::command::error::IoErrorKind>>, bulk::error::Rename> {
    let Some((project_path, project_data)) = db.project().get_by_id(project.clone()).unwrap()
    else {
        return Err(bulk::error::Rename::ProjectNotFound);
    };

    let db::state::DataResource::Ok(properties) = project_data.properties() else {
        panic!("invalid state");
    };
    assert_eq!(properties.rid(), &project);

    let data_root = project_path.join(&properties.data_root);
    let (rename_paths, rename_errors) = containers
        .into_iter()
        .map(|container| {
            let path = db::common::container_system_path(&data_root, &container);
            let mut path_new = path.clone();
            path_new.set_file_name(&name);
            if path_new.exists() {
                Err(container)
            } else {
                Ok((path, path_new))
            }
        })
        .partition::<Vec<_>, _>(|rename| rename.is_ok());

    if rename_errors.len() > 0 {
        let paths = rename_errors
            .into_iter()
            .map(|err| {
                let Err(path) = err else {
                    unreachable!("invalid result");
                };
                path
            })
            .collect();

        return Err(bulk::error::Rename::NameCollision(paths));
    }

    let rename_results = rename_paths
        .into_iter()
        .map(|result| {
            let Ok((from, to)) = result else {
                unreachable!("invalid result");
            };

            fs::rename(from, to).map_err(|err| lib::command::error::IoErrorKind(err.kind()))
        })
        .collect();

    Ok(rename_results)
}

/// Update multiple containers' properties.
#[tauri::command]
pub fn container_properties_update_bulk(
    db: tauri::State<db::Client>,
    project: ResourceId,
    containers: Vec<PathBuf>,
    // update: bulk::PropertiesUpdate,
    update: String, // TODO: Issue with serializing enum with Option. perform manually.
                    // See: https://github.com/tauri-apps/tauri/issues/5993
) -> Result<Vec<Result<(), bulk::error::Update>>, bulk::error::ProjectNotFound> {
    let update = serde_json::from_str::<PropertiesUpdate>(&update).unwrap();
    tracing::debug!(?update);
    let Some((project_path, project_data)) = db.project().get_by_id(project.clone()).unwrap()
    else {
        return Err(bulk::error::ProjectNotFound);
    };

    let db::state::DataResource::Ok(project_properties) = project_data.properties() else {
        panic!("invalid state");
    };
    assert_eq!(project_properties.rid(), &project);

    let data_root = project_path.join(&project_properties.data_root);
    Ok(containers
        .iter()
        .map(|container| {
            let path = db::common::container_system_path(&data_root, container);
            container_properties_update_bulk_perform(&path, &update)
        })
        .collect())
}

fn container_properties_update_bulk_perform(
    path: impl AsRef<Path>,
    update: &bulk::PropertiesUpdate,
) -> Result<(), bulk::error::Update> {
    let mut container =
        match local::loader::container::Loader::load_from_only_properties(path.as_ref()) {
            Ok(container) => container,
            Err(err) => return Err(bulk::error::Update::Load(err)),
        };

    if let Some(kind) = &update.kind {
        container.properties.kind = kind.clone();
    }

    if let Some(description) = &update.description {
        container.properties.description = description.clone();
    }

    container
        .properties
        .tags
        .retain(|tag| !update.tags.remove.contains(tag));
    container.properties.tags.extend(update.tags.insert.clone());

    container
        .properties
        .metadata
        .retain(|key, _| !update.metadata.remove.contains(key));
    container
        .properties
        .metadata
        .extend(update.metadata.insert.clone());

    if let Err(err) = container.save(&path) {
        return Err(bulk::error::Update::Save(err.kind()));
    }

    Ok(())
}
