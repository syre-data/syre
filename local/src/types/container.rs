use serde::{Deserialize, Serialize};
use syre_core::types::UserPermissions;

// *************************
// *** ContainerSettings ***
// *************************

/// Settings for a Container
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ContainerSettings {
    #[serde(default)]
    pub permissions: Vec<UserPermissions>,
}
