//! Container.
use super::asset::Asset;
use super::script_association::RunParameters;
use super::standard_properties::StandardProperties;
use crate::types::{ResourceId, ResourceMap, ResourceStore};
use std::sync::{Arc, Mutex};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// *************
// *** types ***
// *************

pub type ContainerWrapper = Arc<Mutex<Container>>;
pub type ContainerStore = ResourceStore<ContainerWrapper>;
pub type AssetMap = ResourceMap<Asset>;
pub type ScriptMap = ResourceMap<RunParameters>;

// *****************
// *** Container ***
// *****************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Container {
    pub rid: ResourceId,
    pub properties: StandardProperties,
    pub parent: Option<ResourceId>,
    pub children: ContainerStore,
    pub assets: AssetMap,
    pub scripts: ScriptMap,
}

impl Default for Container {
    fn default() -> Container {
        Container {
            rid: ResourceId::new(),
            properties: StandardProperties::default(),
            parent: None,
            children: ContainerStore::default(),
            assets: AssetMap::default(),
            scripts: ScriptMap::default(),
        }
    }
}

impl PartialEq for Container {
    /// Compares `rid` and `properties` for equality.
    /// Ignores all other fields.
    fn eq(&self, other: &Self) -> bool {
        (self.rid == other.rid) && (self.properties == other.properties)
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
