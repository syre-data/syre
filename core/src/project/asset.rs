/// Asset.
use super::{standard_properties::Builder as PropertiesBuilder, Metadata, StandardProperties};
use crate::db::{Resource, StandardResource};
use crate::types::{Creator, ResourceId, ResourcePath};
use chrono::prelude::*;
use has_id::HasId;
use serde_json::Value as JsValue;
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

impl Resource for Asset {}

impl StandardResource for Asset {
    fn properties(&self) -> &StandardProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut StandardProperties {
        &mut self.properties
    }
}

// ***************
// *** Builder ***
// ***************

pub struct NoPath;
pub struct Path(ResourcePath);

pub struct Builder<P> {
    properties: PropertiesBuilder,
    path: P,
}

impl<P> Builder<P> {
    pub fn set_created(&mut self, value: DateTime<Utc>) -> &mut Self {
        self.properties.set_created(value);
        self
    }

    pub fn clear_created(&mut self) -> &mut Self {
        self.properties.clear_created();
        self
    }

    pub fn set_creator(&mut self, value: Creator) -> &mut Self {
        self.properties.set_creator(value);
        self
    }

    pub fn set_name(&mut self, value: String) -> &mut Self {
        self.properties.set_name(value);
        self
    }

    pub fn clear_name(&mut self) -> &mut Self {
        self.properties.clear_name();
        self
    }

    pub fn set_kind(&mut self, value: String) -> &mut Self {
        self.properties.set_kind(value);
        self
    }

    pub fn clear_kind(&mut self) -> &mut Self {
        self.properties.clear_kind();
        self
    }

    pub fn set_description(&mut self, value: String) -> &mut Self {
        self.properties.set_description(value);
        self
    }

    pub fn clear_description(&mut self) -> &mut Self {
        self.properties.clear_description();
        self
    }

    pub fn set_tags(&mut self, value: Vec<String>) -> &mut Self {
        self.properties.set_tags(value);
        self
    }

    pub fn clear_tags(&mut self) -> &mut Self {
        self.properties.clear_tags();
        self
    }

    pub fn add_tag(&mut self, value: impl Into<String>) -> &mut Self {
        self.properties.add_tag(value);
        self
    }

    pub fn remove_tag(&mut self, value: impl Into<String>) -> &mut Self {
        self.properties.remove_tag(value);
        self
    }

    pub fn set_metadata(&mut self, value: Metadata) -> &mut Self {
        self.properties.set_metadata(value);
        self
    }

    pub fn clear_metadata(&mut self) -> &mut Self {
        self.properties.clear_metadata();
        self
    }

    pub fn set_metadatum(
        &mut self,
        key: impl Into<String>,
        value: impl Into<JsValue>,
    ) -> &mut Self {
        self.properties.set_metadatum(key.into(), value.into());
        self
    }

    pub fn remove_metadatum(&mut self, key: impl Into<String>) -> &mut Self {
        self.properties.remove_metadatum(key);
        self
    }

    pub fn set_path(self, value: ResourcePath) -> Builder<Path> {
        Builder {
            properties: self.properties,
            path: Path(value),
        }
    }
}

impl Builder<NoPath> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Builder<Path> {
    pub fn clear_path(self) -> Builder<NoPath> {
        Builder {
            properties: self.properties,
            path: NoPath,
        }
    }
}

impl Default for Builder<NoPath> {
    fn default() -> Self {
        Self {
            properties: PropertiesBuilder::default(),
            path: NoPath,
        }
    }
}

impl Into<Asset> for Builder<Path> {
    fn into(self) -> Asset {
        Asset {
            rid: ResourceId::new(),
            properties: self.properties.into(),
            path: self.path.0,
        }
    }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
