//! Container.
use super::standard_properties::StandardProperties;
use super::{Asset, RunParameters};
use crate::db::{Resource, StandardResource};
use crate::types::{ResourceId, ResourceMap};
use has_id::HasId;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// *************
// *** types ***
// *************

pub type AssetMap = ResourceMap<Asset>;
pub type ScriptMap = ResourceMap<RunParameters>;

// *****************
// *** Container ***
// *****************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[derive(PartialEq, Eq, Clone, Debug, HasId)]
pub struct Container {
    #[id]
    pub rid: ResourceId,
    pub properties: StandardProperties,
    pub assets: AssetMap,
    pub scripts: ScriptMap,
}

impl Default for Container {
    fn default() -> Container {
        Container {
            rid: ResourceId::new(),
            properties: StandardProperties::default(),
            assets: AssetMap::default(),
            scripts: ScriptMap::default(),
        }
    }
}

impl Hash for Container {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rid.hash(state);
    }
}

impl Resource for Container {}

impl StandardResource for Container {
    fn properties(&self) -> &StandardProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut StandardProperties {
        &mut self.properties
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
