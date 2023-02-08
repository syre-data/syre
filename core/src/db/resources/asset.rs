//! Asset
use super::object::{Object, StandardObject};
use super::standard_properties::StandardProperties;
use crate::project::asset::Asset as PrjAsset;
use crate::types::{ResourceId, ResourcePath};
use has_id::{HasId, HasIdMut};
use std::hash::Hash;
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::Deserialize;

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

// @todo: Ensure that if `parent` is set it's id matches that of `parent_id`.
/// Asset
#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[cfg_attr(feature = "serde", derive(Deserialize, HasIdSerde))]
#[derive(HasId, HasIdMut, Hash, Clone, PartialEq, Eq, Debug)]
pub struct Asset {
    #[id]
    pub rid: ResourceId,
    pub properties: StandardProperties,

    /// Path to the asset file.
    pub path: ResourcePath,

    /// [`Container`] the `Asset` belongs to.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub parent: Option<ResourceId>,
}

impl Asset {
    /// Converts a project Asset into a database Asset.
    pub fn from(asset: PrjAsset, container: ResourceId) -> Self {
        Asset {
            rid: asset.rid,
            properties: asset.properties.into(),
            path: asset.path,
            parent: Some(container),
        }
    }

    // @todo: Container could be thought of as a root bucket.
    // If this is the case, should possibly return `std::path::Component::RootDir`
    // or `std::path::component::CurDir` instead of `None`.
    /// Returns the bucket of the `Asset`, or `None` if it is in the root.
    ///
    /// # See also
    /// + [`Self.of_root`]
    /// + [`Self.in_bucket`]
    /// + [`Self.of_bucket`]
    pub fn bucket(&self) -> Option<PathBuf> {
        let ResourcePath::Relative(path) = self.path.clone() else {
            return None;
        };

        path.as_path().parent().map(|p| p.to_path_buf())
    }

    /// Returns if the Asset is in the Container root.
    ///
    /// # See also
    /// + [`Self.bucket`]
    /// + [`Self.in_bucket`]
    /// + [`Self.of_bucket`]
    pub fn of_root(&self) -> bool {
        self.bucket() == Some(PathBuf::from(""))
    }

    /// Returns if the Asset is in the given bucket or one of its descendents.
    ///
    /// # See also
    /// + [`Self.bucket`]
    /// + [`Self.of_root`]
    /// + [`Self.of_bucket`]
    pub fn in_bucket(&self, bucket: &Path) -> bool {
        let Some(my_bucket) = self.bucket() else {
            return false;
        };

        my_bucket.starts_with(bucket)
    }

    /// Returns if the Asset is in the given bucket directly.
    ///
    /// # See also
    /// + [`Self.bucket`]
    /// + [`Self.of_root`]
    /// + [`Self.in_bucket`]
    pub fn of_bucket(&self, bucket: &Path) -> bool {
        let Some(my_bucket) = self.bucket() else {
            return false;
        };

        my_bucket == bucket
    }
}

impl Object for Asset {}

impl StandardObject for Asset {
    fn properties(&self) -> &StandardProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut StandardProperties {
        &mut self.properties
    }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
