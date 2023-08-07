/// Asset and Assets.
use super::standard_properties::StandardProperties;
use crate::common;
use crate::file_resource::LocalResource;
use crate::Result;
use std::fs;
use std::io::BufReader;
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
pub struct Assets {
    base_path: PathBuf,
    assets: AssetMap,
}

impl Assets {
    pub fn load_from(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();
        let path = base_path.join(Self::rel_path());
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let assets = serde_json::from_reader(reader)?;

        Ok(Self { base_path, assets })
    }

    pub fn save(&self) -> Result {
        let file = fs::OpenOptions::new().write(true).open(self.path())?;
        Ok(serde_json::to_writer_pretty(file, &self.assets)?)
    }
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

impl LocalResource<AssetMap> for Assets {
    fn rel_path() -> PathBuf {
        common::assets_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
