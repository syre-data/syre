use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_local::types::FsResourceAction;

/// Info for adding a file system resource to a data graph.
///
/// # Notes
/// + Paths should be absolute from the file system root.
#[derive(Serialize, Deserialize)]
pub struct AddFsResourceData {
    pub path: PathBuf,
    pub parent: PathBuf,
    pub action: FsResourceAction,
}
