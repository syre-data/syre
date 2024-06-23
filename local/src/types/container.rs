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

    #[serde(default)]
    pub permissions: Vec<UserPermissions>,
}
