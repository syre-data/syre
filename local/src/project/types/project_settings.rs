//! Local `Project` settings.
use super::super::PROJECT_FORMAT;
use serde::{Deserialize, Serialize};
use thot_core::types::{ResourceMap, UserPermissions};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ProjectSettings {
    /// Format standard for the Project.
    pub local_format_standard: String,
    pub permissions: ResourceMap<UserPermissions>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            local_format_standard: PROJECT_FORMAT.to_string(),
            permissions: ResourceMap::default(),
        }
    }
}

#[cfg(test)]
#[path = "./project_settings_test.rs"]
mod project_settings_test;
