/// Asset and Assets.
use super::standard_properties::StandardProperties;
use crate::common::assets_file_of;
use crate::error::{AssetError, Result};
use cluFlock::FlockLock;
use serde::{Deserialize, Serialize};
use settings_manager::error::{
    Error as SettingsError, Result as SettingsResult, SettingsError as LocalSettingsError,
};
use settings_manager::local_settings::{LocalSettings, LockSettingsFile};
use settings_manager::settings::Settings;
use settings_manager::types::Priority as SettingsPriority;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::project::container::AssetMap;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::resource_map::values_only;
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
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Assets {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _base_path: Option<PathBuf>,

    #[serde(with = "values_only")]
    assets: AssetMap,
}

impl Assets {
    pub fn new() -> Self {
        Assets {
            _file_lock: None,
            _base_path: None,

            assets: AssetMap::new(),
        }
    }

    /// Inserts an [`Asset`].
    ///
    /// # Returns
    /// The existing value of the [`Asset`] if present, or
    /// `None` otherwise.
    ///
    /// # Errors
    /// + [`AssetError::FileAlreadyAsset`]: If an [`Asset`] referencing
    ///    the same file is already present.
    pub fn insert_asset(&mut self, asset: CoreAsset) -> Result<Option<CoreAsset>> {
        if self.get_path(&asset.path).is_some() {
            // file is already registered
            return Err(AssetError::FileAlreadyAsset(asset.path.as_path().to_path_buf()).into());
        }

        Ok(self.assets.insert(asset.rid.clone(), asset))
    }

    /// Returns the [`Asset`](CoreAsset)s with the given path if
    /// it exists, otherwise `None`.
    ///
    /// # Notes
    /// + Does not enforce `Asset` file uniqueness.
    ///    If multiple `Asset`s reference the same file,
    ///    the first encountered is returned.
    pub fn get_path(&self, path: &ResourcePath) -> Option<&CoreAsset> {
        for asset in self.assets.values() {
            if &asset.path == path {
                return Some(&asset);
            }
        }

        None
    }
}

impl Settings for Assets {
    fn store_lock(&mut self, lock: FlockLock<File>) {
        self._file_lock = Some(lock);
    }

    fn file(&self) -> Option<&File> {
        match self._file_lock.as_ref() {
            None => None,
            Some(lock) => Some(&*lock),
        }
    }

    fn file_mut(&mut self) -> Option<&mut File> {
        match self._file_lock.as_mut() {
            None => None,
            Some(lock) => Some(lock),
        }
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings for Assets {
    fn rel_path() -> SettingsResult<PathBuf> {
        Ok(assets_file_of(Path::new("")))
    }

    fn base_path(&self) -> SettingsResult<PathBuf> {
        self._base_path
            .clone()
            .ok_or(SettingsError::SettingsError(LocalSettingsError::PathNotSet))
    }

    fn set_base_path(&mut self, path: PathBuf) -> SettingsResult {
        self._base_path = Some(path);
        Ok(())
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
impl LockSettingsFile for Assets {}

#[cfg(test)]
#[path = "./asset_test.rs"]
mod asset_test;
