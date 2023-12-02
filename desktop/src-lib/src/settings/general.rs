//! General settings for the desktop app.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_local::types::AssetFileAction;

#[derive(Serialize, Deserialize, PartialEq, Clone, Default, Debug)]
pub struct GeneralSettings {
    pub default_project_directory: Option<PathBuf>,

    #[serde(default)]
    pub ondrop_asset_action: AssetFileAction,

    #[serde(default)]
    pub open_previous_project_on_start: bool,

    #[serde(default)]
    pub rename_container_folder: bool,
}

impl GeneralSettings {
    pub fn new() -> Self {
        Self {
            default_project_directory: None,
            open_previous_project_on_start: true,
            ondrop_asset_action: AssetFileAction::Copy,
            rename_container_folder: true,
        }
    }
}
