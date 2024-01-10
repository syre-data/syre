//! Resources for [`graph commands`](thot_desktop_tauri::commands::graph).
use super::common::ResourceIdArgs;
use crate::common::invoke_result;
use serde::Serialize;
use std::path::PathBuf;
use thot_core::graph::ResourceTree;
use thot_core::project::Container;
use thot_core::types::ResourceId;
use thot_local_database::error::server::LoadProjectGraph;
use thot_local_database::Result as DbResult;

type ContainerTree = ResourceTree<Container>;

pub async fn init_project_graph(
    project: ResourceId,
    path: PathBuf,
) -> Result<ContainerTree, String> {
    invoke_result("init_project_graph", InitProjectGraphArgs { path, project }).await
}

pub async fn load_project_graph(project: ResourceId) -> Result<ContainerTree, LoadProjectGraph> {
    invoke_result::<ContainerTree, LoadProjectGraph>(
        "load_project_graph",
        ResourceIdArgs { rid: project },
    )
    .await
}

pub async fn duplicate_container_tree(root: ResourceId) -> DbResult<ContainerTree> {
    invoke_result("duplicate_container_tree", ResourceIdArgs { rid: root }).await
}

pub async fn remove_container_tree(root: ResourceId) -> Result<(), String> {
    invoke_result("remove_container_tree", ResourceIdArgs { rid: root }).await
}

pub async fn new_child(name: String, parent: ResourceId) -> Result<(), String> {
    invoke_result("new_child", NewChildArgs { name, parent }).await
}

/// Arguments for [`init_project_graph`](thot_desktop_tauri::commands::graph::init_project_graph).
#[derive(Serialize)]
pub struct InitProjectGraphArgs {
    /// Path to use as data root.
    pub path: PathBuf,

    /// Project id.
    pub project: ResourceId,
}

/// Arguments for [`new_child`](thot_desktop_tauri::commands::container::new_child).
#[derive(Serialize)]
pub struct NewChildArgs {
    /// Name of the child.
    pub name: String,

    /// [`ResourceId`] of the parent [`Container`](thot_core::project::Container).
    pub parent: ResourceId,
}
