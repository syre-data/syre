//! Graph commands.
use crate::error::{DesktopSettingsError, Result};
use crate::state::AppState;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::project::{Container, Project};
use thot_core::types::{Creator, ResourceId, UserId};
use thot_desktop_lib::error::{Error as LibError, Result as LibResult};
use thot_local::project::container;
use thot_local_database::client::Client as DbClient;
use thot_local_database::command::container::UpdatePropertiesArgs;
use thot_local_database::command::graph::NewChildArgs;
use thot_local_database::command::{ContainerCommand, GraphCommand, ProjectCommand};
use thot_local_database::Result as DbResult;

type ContainerTree = ResourceTree<Container>;

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

    let user = UserId::Id(user.rid.clone());
    let mut container = Container::new();
    container.properties.creator = Creator::User(Some(user));

    // create data folder
    fs::create_dir(&path).expect("could not create data root directory");
    let _rid = container::init_from(&path, container)?;

    // set project data root
    let project = db
        .send(ProjectCommand::Get(project).into())
        .expect("could not get `Project`");

    let project: Option<Project> =
        serde_json::from_value(project).expect("could not convert `Get` result to `Project`");

    let Some(mut project) = project else {
        return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` not loaded")).into());
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
pub fn load_project_graph(db: State<DbClient>, rid: ResourceId) -> LibResult<ContainerTree> {
    let graph = db
        .send(GraphCommand::Load(rid).into())
        .expect("could not load graph");

    let graph: DbResult<ContainerTree> = serde_json::from_value(graph)
        .expect("could not convert `Load` result to a `ContainerTree`");

    let graph = graph.map_err(|err| LibError::Database(format!("{err:?}")))?;
    Ok(graph)
}

/// Creates a new child [`Container`](LocalContainer).
/// Adds the child into the [`ContainerStore`].
///
/// # Arguments
/// 1. `name`: Name of the child.
/// 2. `parent`: [`ResourceId`] of the parent [`Container`](LocalContainer).
#[tracing::instrument(level = "debug", skip(db))]
#[tauri::command]
pub fn new_child(db: State<DbClient>, name: String, parent: ResourceId) -> Result<Container> {
    let child = db
        .send(GraphCommand::NewChild(NewChildArgs { name, parent }).into())
        .expect("could not create child `Container`");

    let child: DbResult<Container> = serde_json::from_value(child)
        .expect("could not convert `NewChild` result to a `Container`");

    Ok(child?)
}

/// Duplicates a [`Container`](LocalContainer) tree.
///
/// # Arguments
/// 1. Id of the root of the `Container` tree to duplicate.
#[tracing::instrument(skip(db))]
#[tauri::command]
pub fn duplicate_container_tree(db: State<DbClient>, rid: ResourceId) -> LibResult<ContainerTree> {
    let dup = db
        .send(GraphCommand::Duplicate(rid).into())
        .expect("could not duplicate graph");

    let dup: DbResult<ContainerTree> = serde_json::from_value(dup)
        .expect("could not convert result of `Dupilcate` to `Container` tree");

    // Update name
    let mut dup = dup.map_err(|err| LibError::Database(format!("{err:?}")))?;
    let root_id = dup.root().clone();
    let root = dup
        .get_mut(&root_id)
        .expect("duplicated tree root not found");

    let name = match root.properties.name.clone() {
        None => "Copy".to_string(),
        Some(mut name) => {
            // TODO[m]: Add index to copy, instead of new "(Copy)" every time.
            name.push_str(" (Copy)");
            name
        }
    };

    root.properties.name = Some(name);

    let res = db
        .send(
            ContainerCommand::UpdateProperties(UpdatePropertiesArgs {
                rid: root_id,
                properties: root.properties.clone(),
            })
            .into(),
        )
        .expect("could not update root `Container`");

    let res: DbResult = serde_json::from_value(res)
        .expect("could not convert result of `UpdateContainerProperties` from JsValue");

    res.map_err(|err| LibError::Database(format!("{err:?}")))?;
    Ok(dup)
}

/// Removes a [`Container`](LocalContainer) tree.
///
/// # Arguments
/// 1. Id of the root of the `Container` tree to remove.
#[tauri::command]
pub fn remove_container_tree(db: State<DbClient>, rid: ResourceId) -> Result {
    let res = db
        .send(GraphCommand::Remove(rid).into())
        .expect("could not remove `Container` tree");
    let res: DbResult = serde_json::from_value(res)
        .expect("could not convert result of `Remove` to `Container` tree");
    res.map_err(|err| err.into())
}
