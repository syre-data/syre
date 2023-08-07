use serde::{Deserialize, Serialize};
use thot_core::project::{container::ScriptMap, Container as CoreContainer, StandardProperties};
use thot_core::types::{ResourceId, UserPermissions};

// *************************
// *** ContainerSettings ***
// *************************

/// Settings for a Container
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ContainerSettings {
    pub permissions: Vec<UserPermissions>,
}

// ***************************
// *** ContainerProperties ***
// ***************************

/// Properties for a Container.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerProperties {
    pub rid: ResourceId,
    pub properties: StandardProperties,
    pub scripts: ScriptMap,
}

impl From<CoreContainer> for ContainerProperties {
    fn from(container: CoreContainer) -> Self {
        Self {
            rid: container.rid,
            properties: container.properties,
            scripts: container.scripts,
        }
    }
}
