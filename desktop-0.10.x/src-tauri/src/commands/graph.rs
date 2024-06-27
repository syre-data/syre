//! Graph commands.
use super::utils;
use crate::error::{DesktopSettings as DesktopSettingsError, Error, Result};
use crate::state::AppState;
use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::graph::ResourceTree;
use syre_core::project::Container as CoreContainer;
use syre_core::types::{Creator, ResourceId, UserId};
use syre_desktop_lib::error::{RemoveResource as RemoveResourceError, Trash as TrashError};
use syre_local::error::{ContainerError, Error as LocalError};
use syre_local::project::resources::Container as LocalContainer;
use syre_local_database::client::Client as DbClient;
use syre_local_database::error::server::LoadProjectGraph as LoadProjectGraphError;
use tauri::State;

type ContainerTree = ResourceTree<CoreContainer>;

/// Initializes a directory as a [`Container`](LocalContainer).
/// Sets the Project's `data_root` to the path.
///
/// # Arguments
/// 1. `project`: Id of the project to initialize the graph for.
/// 2. `path`: [`Container`](Container) to initialize and set as data_root.
///
/// # Returns
/// Initial container tree.
#[tauri::command]
pub fn init_project_graph(
    db: State<DbClient>,
    app_state: State<AppState>,
    project: ResourceId,
    path: PathBuf,
) -> Result<ContainerTree> {
    let user = app_state
        .user
        .lock()
        .expect("could not lock `AppState.user`");

    let Some(user) = user.as_ref() else {
        return Err(DesktopSettingsError::NoUser.into());
    };

    // create data folder
    let user = UserId::Id(user.rid.clone());
    let mut container = LocalContainer::new(path.clone());
    container.properties.creator = Creator::User(Some(user));
    container.save()?;

    // set project data root
    let Some(mut project) = db.project().get(project).unwrap() else {
        return Err(
            CoreError::Resource(ResourceError::does_not_exist("`Project` not loaded")).into(),
        );
    };

    project.data_root = path.clone();
    let pid = project.rid.clone();
    if let Err(err) = db.project().update(project).unwrap() {
        return Err(Error::LocalDatabaseError(err.into()));
    }

    db.graph()
        .get_or_load(pid)
        .unwrap()
        .map_err(|err| Error::LocalDatabaseError(err.into()))
}

/// Loads a Container graph from path.
///
/// # Argments
/// 1. `Project` id.
#[tauri::command]
pub fn load_project_graph(
    db: State<DbClient>,
    rid: ResourceId,
) -> StdResult<ContainerTree, LoadProjectGraphError> {
    db.graph().load(rid).unwrap()
}

/// Gets a Container graph from path, loading it if needed.
///
/// # Argments
/// 1. `Project` id.
#[tauri::command]
pub fn get_or_load_project_graph(
    db: State<DbClient>,
    rid: ResourceId,
) -> StdResult<ContainerTree, LoadProjectGraphError> {
    db.graph().get_or_load(rid).unwrap()
}

/// Creates a new child [`Container`](LocalContainer).
///
/// # Arguments
/// 1. `name`: Name of the child.
/// 2. `parent`: [`ResourceId`] of the parent [`Container`](LocalContainer).
#[tauri::command]
pub fn new_child(db: State<DbClient>, name: String, parent: ResourceId) -> Result {
    let Some(mut path) = db.container().path(parent).unwrap() else {
        panic!("could not get container path");
    };

    path.push(name);
    if path.exists() {
        return Err(LocalError::ContainerError(ContainerError::InvalidChildPath(path)).into());
    }

    fs::create_dir(path).unwrap();
    Ok(())
}

/// Duplicates a [`Container`](LocalContainer) tree.
///
/// # Arguments
/// 1. Id of the root of the `Container` tree to duplicate.
#[tauri::command]
pub fn duplicate_container_tree(db: State<DbClient>, rid: ResourceId) -> Result<ContainerTree> {
    Ok(db.graph().duplicate(rid).unwrap()?)
}

/// Removes a [`Container`](LocalContainer) tree.
///
/// # Arguments
/// 1. `rid`: Id of the root of the `Container` tree to remove.
#[tauri::command]
pub fn remove_container_tree(
    db: State<DbClient>,
    rid: ResourceId,
) -> StdResult<(), RemoveResourceError> {
    let remove_container_from_db =
        |container: ResourceId| -> StdResult<PathBuf, RemoveResourceError> {
            todo!();
            // match db.graph().remove(container).unwrap() {
            //     Ok(Some((_asset, path))) => Ok(path),
            //     Ok(None) => {
            //         return Err(RemoveResourceError::Database(
            //             "container does not exist".to_string(),
            //         ))
            //     }
            //     Err(err) => return Err(RemoveResourceError::Database(format!("{err:?}"))),
            // }
        };

    let Some(path) = db.container().path(rid.clone()).unwrap() else {
        return Err(RemoveResourceError::Database(
            "Could not get Container's path".to_string(),
        ));
    };

    match trash::delete(path) {
        Ok(_) => Ok(()),

        Err(trash::Error::CanonicalizePath { original: _ }) => {
            match remove_container_from_db(rid) {
                Ok(_) => Err(TrashError::NotFound.into()),
                Err(err) => Err(err),
            }
        }

        Err(trash::Error::CouldNotAccess { target }) => {
            if Path::new(&target).exists() {
                Err(TrashError::PermissionDenied.into())
            } else {
                match remove_container_from_db(rid) {
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
                    remove_container_from_db(rid)?;
                    Err(TrashError::NotFound.into())
                }

                err => Err(err.into()),
            }
        }

        Err(trash::Error::Unknown { description }) => Err(TrashError::Other(description).into()),

        Err(err) => Err(TrashError::Other(format!("{err:?}")).into()),
    }
}
