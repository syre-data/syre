use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_local::types::AssetFileAction;

// TODO Merge with `syre_local::types::AssetFileAction`.
/// Info for adding an [`Asset`](syre_core::project::Asset).
#[derive(Serialize, Deserialize, Debug)]
pub struct AddAssetInfo {
    /// Path of the file to make an [`Asset`](syre_core::project::Asset).
    pub path: PathBuf,

    /// How to handle the file on disk.
    pub action: AssetFileAction,

    /// The bucket to place the [`Asset`](syre_core::project::Asset)'s file in.
    pub bucket: Option<PathBuf>,
}
