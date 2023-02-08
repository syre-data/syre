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
    pub children: ContainerStore,
    pub assets: AssetMap,
    pub scripts: ScriptMap,
}

impl Container {
    pub fn new() -> Container {
        Container {
            rid: ResourceId::new(),
            properties: StandardProperties::new(),
            children: ContainerStore::new(),
            assets: AssetMap::new(),
            scripts: ScriptMap::new(),
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
