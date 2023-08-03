//! Container types.
use serde::{Deserialize, Serialize};
use thot_core::project::container::{Container as CoreContainer, ScriptMap};
use thot_core::project::StandardProperties;
use thot_core::types::{ResourceId, UserPermissions};

// ****************************
// *** Container Properties ***
// ****************************

/// Container properties for persistance.
#[derive(Serialize, Deserialize, Clone)]
pub struct ContainerProperties {
    pub rid: ResourceId,
    pub properties: StandardProperties,
    pub scripts: ScriptMap,
}

impl ContainerProperties {
    pub fn scripts_mut(&mut self) -> &mut ScriptMap {
        &mut self.scripts
    }
}

impl Default for ContainerProperties {
    fn default() -> Self {
        Self {
            rid: ResourceId::new(),
            properties: StandardProperties::default(),
            scripts: ScriptMap::default(),
        }
    }
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

// **************************
// *** Container Settings ***
// **************************

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ContainerSettings {
    pub permissions: Vec<UserPermissions>,
}
