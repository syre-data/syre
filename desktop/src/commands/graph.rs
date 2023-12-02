//! Resources for [`graph commands`](thot_desktop_tauri::commands::graph).
use serde::Serialize;
use std::path::PathBuf;
use thot_core::types::ResourceId;

/// Arguments for [`init_project_graph`](thot_desktop_tauri::commands::graph::init_project_graph).
#[derive(Serialize)]
pub struct InitProjectGraphArgs {
    /// Path to use as data root.
    pub path: PathBuf,

    /// Project id.
    pub project: ResourceId,
}
