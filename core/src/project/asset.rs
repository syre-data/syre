/// Asset.
use super::standard_properties::StandardProperties;
// use crate::db::{Resource, StandardResource};
use crate::types::{ResourceId, ResourcePath};
use has_id::HasId;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

/// Assets represent a consumable or producable resource.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, HasIdSerde))]
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[derive(HasId, Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    #[id]
    pub rid: ResourceId,
    pub properties: StandardProperties,

    /// Path to the `Asset`'s resource file.
    pub path: ResourcePath,
}

impl Asset {
    pub fn new(path: ResourcePath) -> Asset {
        Asset {
            rid: ResourceId::new(),
            properties: StandardProperties::new(),
            path,
        }
    }

    /// Returns the `bucket` path of the `Asset`
    /// if it is in one, otherwise `None`.
    ///
    /// # Notes
    /// + An `Asset` is in a `bucket` if its `path` is
    /// a [`ResourcePath::Relative`] with no ancestors (i.e. `..`)
    /// after canonicalization.
    /// + An `Asset` in its [`Container`](super::Container)'s root
    /// is considered to be in the `root` `bucket`.
    pub fn bucket(&self) -> Option<PathBuf> {
        let ResourcePath::Relative(path) = self.path.clone() else {
            return None;
        };

        path.parent().map(|p| p.to_path_buf())
    }
}

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rid.hash(state);
    }
}

// impl Resource for Asset {}

// impl StandardResource for Asset {
//     fn properties(&self) -> &StandardProperties {
//         &self.properties
//     }

//     fn properties_mut(&mut self) -> &mut StandardProperties {
//         &mut self.properties
//     }
// }

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
