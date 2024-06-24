use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use syre_core::types::{UserId, UserPermissions};

// *************************
// *** ContainerSettings ***
// *************************

/// Settings for a Container
#[derive(PartialEq, Serialize, Deserialize, Clone, Default, Debug)]
pub struct ContainerSettings {
    pub creator: Option<UserId>,
    pub created: DateTime<Utc>,
    pub permissions: Vec<UserPermissions>,
}

impl ContainerSettings {
    pub fn new() -> Self {
        Self {
            creator: None,
            created: Utc::now(),
            permissions: vec![],
        }
    }
}
