//! Container.
use super::standard_properties::StandardProperties;
use super::{Asset, RunParameters};
use crate::db::{Resource, StandardResource};
use crate::error::{ResourceError, Result};
use crate::types::{ResourceId, ResourceMap, ResourceStore};
use has_id::HasId;
use std::hash::{Hash, Hasher};
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
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[derive(Clone, Debug, HasId)]
pub struct Container {
    #[id]
    pub rid: ResourceId,
    pub properties: StandardProperties,
    pub parent: Option<ResourceId>,
    pub children: ContainerStore,
    pub assets: AssetMap,
    pub scripts: ScriptMap,
}

impl Container {
    /// Duplicates the tree.
    /// Copies the structure of the tree,
    /// along with `properties`, and `scripts`,
    /// into a new tree.
    pub fn duplicate(&self) -> Result<Self> {
        let mut dup = Self::default();
        dup.properties = self.properties.clone();
        dup.scripts = self.scripts.clone();
        dup.parent = self.parent.clone();

        for child in self.children.clone().values() {
            let Some(child) = child else {
                return Err(ResourceError::DoesNotExist("child `Container` not loaded".to_string()).into());
            };

            let child = child.lock().expect("could not lock child `Container`");
            let mut dup_child = child.duplicate()?;

            dup_child.parent = Some(dup.rid.clone());
            dup.children
                .insert(dup_child.rid.clone(), Some(Arc::new(Mutex::new(dup_child))));
        }

        Ok(dup)
    }
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
    fn eq(&self, other: &Self) -> bool {
        if self.rid != other.rid {
            return false;
        }

        if self.properties != other.properties {
            return false;
        }

        true
    }
}

impl Eq for Container {}

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
