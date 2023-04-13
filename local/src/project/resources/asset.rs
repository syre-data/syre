/// Asset and Assets.
use super::standard_properties::StandardProperties;
use crate::common::assets_file;
use crate::Result;
use cluFlock::FlockLock;
use settings_manager::local_settings::{Components, Loader, LocalSettings};
use settings_manager::Settings;
use std::borrow::Cow;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::container::AssetMap;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::{ResourceId, ResourcePath};

// *******************
// *** Local Asset ***
// *******************

pub struct Asset;

impl Asset {
    /// Creates an [] with the `properties` field filled actively from
    /// [`LocalStandardProperties`].
    pub fn new(path: ResourcePath) -> Result<CoreAsset> {
        let props = StandardProperties::new()?;
        Ok(CoreAsset {
            rid: ResourceId::new(),
            properties: props,
            path,
        })
    }
}

// **************
// *** Assets ***
// **************

/// Assets for a given [`Container`].
///
/// # Notes
/// + A [`Container`] may only reference a file in a single [`Asset`].
/// This functionality is enforced in the `insert_asset` method, which
/// should be prefered over `insert`.
#[derive(Settings)]
pub struct Assets {
    #[settings(file_lock = "AssetMap")]
    file_lock: FlockLock<File>,
    base_path: PathBuf,

    #[settings(priority = "Local")]
    assets: AssetMap,
}

impl Deref for Assets {
    type Target = AssetMap;

    fn deref(&self) -> &Self::Target {
        &self.assets
    }
}

impl DerefMut for Assets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.assets
    }
}

impl LocalSettings<AssetMap> for Assets {
    fn rel_path() -> PathBuf {
        assets_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl From<Loader<AssetMap>> for Assets {
    fn from(loader: Loader<AssetMap>) -> Self {
        let loader: Components<AssetMap> = loader.into();
        Self {
            file_lock: loader.file_lock,
            base_path: loader.base_path,
            assets: loader.data,
        }
    }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
