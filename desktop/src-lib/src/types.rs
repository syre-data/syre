use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_local::types::AssetFileAction;

// TODO Merge with `thot_local::types::AssetFileAction`.
/// Info for adding an [`Asset`](thot_core::project::Asset).
#[derive(Serialize, Deserialize, Debug)]
pub struct AddAssetInfo {
    /// Path of the file to make an [`Asset`](thot_core::project::Asset).
    pub path: PathBuf,

    /// How to handle the file on disk.
    pub action: AssetFileAction,

    /// The bucket to place the [`Asset`](thot_core::project::Asset)'s file in.
    pub bucket: Option<PathBuf>,
}
