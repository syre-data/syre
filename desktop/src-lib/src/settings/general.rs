//! General settings for the desktop app.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_local::types::AssetFileAction;

#[derive(Serialize, Deserialize, PartialEq, Clone, Default, Debug)]
pub struct GeneralSettings {
    pub default_project_directory: Option<PathBuf>,
    pub open_previous_project_on_start: bool,
    pub ondrop_asset_action: AssetFileAction,
}

impl GeneralSettings {
    pub fn new() -> Self {
        Self {
            default_project_directory: None,
            open_previous_project_on_start: true,
            ondrop_asset_action: AssetFileAction::Copy,
        }
    }
}

#[cfg(test)]
#[path = "./general_test.rs"]
mod general_test;
