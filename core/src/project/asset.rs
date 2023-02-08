/// Asset.
use super::standard_properties::StandardProperties;
use crate::types::{ResourceId, ResourcePath};
use has_id::HasId;
use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

/// Assets represent a consumable or producable resource.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, HasIdSerde))]
#[derive(HasId, Debug, Clone, PartialEq)]
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

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
