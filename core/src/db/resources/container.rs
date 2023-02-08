//! Container
use super::object::{Object, StandardObject};
use super::standard_properties::StandardProperties;
use crate::types::ResourceId;
use has_id::{HasId, HasIdMut};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::Deserialize;

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

// *****************
// *** Container ***
// *****************

/// Container
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[cfg_attr(feature = "serde", derive(Deserialize, HasIdSerde))]
#[derive(HasId, HasIdMut, Clone, PartialEq, Eq, Debug)]
pub struct Container {
    #[id]
    pub rid: ResourceId,
    pub properties: StandardProperties,
    pub children: HashSet<ResourceId>,
    pub assets: HashSet<ResourceId>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub parent: Option<ResourceId>,
}

impl Container {}

impl Hash for Container {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.properties.hash(state);
    }
}

impl Object for Container {}

impl StandardObject for Container {
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
