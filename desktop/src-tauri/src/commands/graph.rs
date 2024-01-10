//! Graph commands.
use crate::error::{DesktopSettings as DesktopSettingsError, Result};
use crate::state::AppState;
use std::fs;
use std::path::PathBuf;
use std::result::Result as StdResult;
use tauri::State;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::project::{Container as CoreContainer, Project};
use thot_core::types::{Creator, ResourceId, UserId};
use thot_local::error::{ContainerError, Error as LocalError};
use thot_local::project::resources::Container as LocalContainer;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::{ContainerCommand, GraphCommand, ProjectCommand};
use thot_local_database::Result as DbResult;

type ContainerTree = ResourceTree<CoreContainer>;

/// Initializes a directory as a [`Container`](LocalContainer).
///
/// # Argument
/// 1. `path`: Path to the desired child directory.
/// 2. `container`: [`Container`](Container) to initialize with.
///     The [`ResourceId`] is ignored.
///
/// # Returns
/// [`ResourceId`] of the initialized [`Container`](Container).
///
/// # See also
/// + [`thot_local::project::container::init`] for details.
#[tauri::command]
pub fn init_project_graph(
    db: State<DbClient>,
    app_state: State<AppState>,
    project: ResourceId,
    path: PathBuf,
) -> Result<ContainerTree> {
    // create container
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
    let project = db
        .send(ProjectCommand::Get(project).into())
        .expect("could not get `Project`");

    let project: Option<Project> =
        serde_json::from_value(project).expect("could not convert `Get` result to `Project`");

    let Some(mut project) = project else {
        return Err(CoreError::ResourceError(ResourceError::does_not_exist(
            "`Project` not loaded",
        ))
        .into());
    };

    project.data_root = Some(path.clone());
    let pid = project.rid.clone();
    let res = db
        .send(ProjectCommand::Update(project).into())
        .expect("could not update `Project`");

    let res: DbResult =
        serde_json::from_value(res).expect("could not convert `Update` result to `Result");

    res?;

    // load and store container
    let graph = db
        .send(GraphCommand::Load(pid).into())
        .expect("could not load graph");

    let graph: DbResult<ContainerTree> =
        serde_json::from_value(graph).expect("could not convert `Load` result to `Container` tree");

    Ok(graph?)
}

/// Loads a [`Container`]Tree from path.
/// Adds containers into the [`ContainerStore`].
///
/// # Argments
/// 1. `Project` id.
#[tracing::instrument(level = "debug", skip(db))]
#[tauri::command]
pub fn load_project_graph(
    db: State<DbClient>,
    rid: ResourceId,
) -> StdResult<ContainerTree, thot_local_database::error::server::LoadProjectGraph> {
    let res = db.send(GraphCommand::Load(rid).into()).unwrap();
    serde_json::from_value(res).unwrap()
}

/// Creates a new child [`Container`](LocalContainer).
/// Adds the child into the [`ContainerStore`].
///
/// # Arguments
/// 1. `name`: Name of the child.
/// 2. `parent`: [`ResourceId`] of the parent [`Container`](LocalContainer).
#[tracing::instrument(level = "debug", skip(db))]
#[tauri::command]
pub fn new_child(db: State<DbClient>, name: String, parent: ResourceId) -> Result {
    let path = db.send(ContainerCommand::Path(parent).into()).unwrap();
    let Some(mut path) = serde_json::from_value::<Option<PathBuf>>(path).unwrap() else {
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
#[tracing::instrument(skip(db))]
#[tauri::command]
pub fn duplicate_container_tree(db: State<DbClient>, rid: ResourceId) -> DbResult<ContainerTree> {
    let dup = db
        .send(GraphCommand::Duplicate(rid).into())
        .expect("could not duplicate graph");

    serde_json::from_value(dup).unwrap()
}

/// Removes a [`Container`](LocalContainer) tree.
///
/// # Arguments
/// 1. Id of the root of the `Container` tree to remove.
#[tauri::command]
pub fn remove_container_tree(db: State<DbClient>, rid: ResourceId) -> Result {
    let path = db.send(ContainerCommand::Path(rid).into()).unwrap();
    let Some(path) = serde_json::from_value::<Option<PathBuf>>(path).unwrap() else {
        panic!("could not get container path");
    };

    trash::delete(path).unwrap();
    Ok(())
}
