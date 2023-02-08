//! User settings related to default actions.
use crate::types::AssetFileAction;
use serde::{Deserialize, Serialize};

// ************************
// *** User Preferences ***
// ************************

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserPreferences {
    project: ProjectUserPreferences,
    analysis: AnalysisUserPreferences,
}

impl UserPreferences {
    pub fn new() -> Self {
        UserPreferences {
            project: ProjectUserPreferences::new(),
            analysis: AnalysisUserPreferences::new(),
        }
    }
}

// ********************************
// *** Project User Preferences ***
// ********************************

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectUserPreferences {
    asset_file_action: AssetFileAction, // how asset files should be manipulated when changed into an asset
    rename_folder_on_name_change: bool, // rename container flder when name is changed
    delete_on_exclude: bool,            // delete folder and files when a subtree is excluded
    format_metdata_objects: bool,       // pretty print and validate metadata objects in the editor
    show_inherited_metadata: bool,      // show inherited metadata when viewing object details
}

impl ProjectUserPreferences {
    /// Create a new project preferences with default values.
    ///
    /// + **asset_file_action:** AssetFileAction::Move
    /// + **rename_folder_on_name_change:** true
    /// + **delete_on_exclude:** false
    /// + **format_metdata_objects:** true
    /// + **show_inherited_metadata:** false
    pub fn new() -> Self {
        ProjectUserPreferences {
            asset_file_action: AssetFileAction::Move,
            rename_folder_on_name_change: true,
            delete_on_exclude: false,
            format_metdata_objects: true,
            show_inherited_metadata: false,
        }
    }
}

// *********************************
// *** Analysis User Preferences ***
// *********************************

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalysisUserPreferences {}

impl AnalysisUserPreferences {
    pub fn new() -> Self {
        AnalysisUserPreferences {}
    }
}

#[cfg(test)]
#[path = "./user_preferences_test.rs"]
mod user_preferences_test;
