use crate::constants::PROJECT_FORMAT_VERSION;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use syre_core::types::{ResourceMap, UserId, UserPermissions};

/// Settings for a local Project.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ProjectSettings {
    /// Format standard for the Project.
    pub local_format_version: String,
    pub created: DateTime<Utc>,
    pub creator: Option<UserId>,
    pub permissions: ResourceMap<UserPermissions>,
}

impl ProjectSettings {
    pub fn new() -> Self {
        Self {
            local_format_version: PROJECT_FORMAT_VERSION.to_string(),
            created: Utc::now(),
            creator: None,
            permissions: ResourceMap::default(),
        }
    }
}
