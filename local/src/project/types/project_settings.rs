//! Local `Project` settings.
use super::super::PROJECT_FORMAT_VERSION;
use serde::{Deserialize, Serialize};
use thot_core::types::{ResourceMap, UserPermissions};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ProjectSettings {
    /// Format standard for the Project.
    pub local_format_version: String,
    pub permissions: ResourceMap<UserPermissions>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            local_format_version: PROJECT_FORMAT_VERSION.to_string(),
            permissions: ResourceMap::default(),
        }
    }
}
