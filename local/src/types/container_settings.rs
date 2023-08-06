use serde::{Deserialize, Serialize};
use thot_core::types::UserPermissions;

/// Settings for a Container
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ContainerSettings {
    pub permissions: Vec<UserPermissions>,
}
