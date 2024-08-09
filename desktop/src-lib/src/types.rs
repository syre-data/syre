use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_local::types::FsResourceAction;

/// Info for adding a file system resource to the analysis of a project.
///
/// # Notes
/// + `path` should be absolute from the file system root.
/// + `parent` should be absolute from the analysis root.
///     i.e. The analysis root has path `/`.
#[derive(Serialize, Deserialize)]
pub struct AddFsAnalysisResourceData {
    pub path: PathBuf,
    pub parent: PathBuf,
    pub action: FsResourceAction,
}

/// Info for adding a file system resource to a data graph.
///
/// # Notes
/// + `path` and `parent` should be absolute from the file system root.
#[derive(Serialize, Deserialize)]
pub struct AddFsGraphResourceData {
    pub path: PathBuf,
    pub parent: PathBuf,
    pub action: FsResourceAction,
}
