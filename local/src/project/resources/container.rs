//! Container and container settings.
use crate::common::{assets_file, container_file, container_settings_file};
use crate::error::{Error, Result};
use crate::file_resource::LocalResource;
use crate::types::ContainerSettings;
use has_id::HasId;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::container::AssetMap;
use thot_core::project::{Asset, Container as CoreContainer, ScriptAssociation};
use thot_core::types::ResourceId;

// *****************
// *** Container ***
// *****************

pub struct Container {
    base_path: PathBuf,
    container: CoreContainer,
    settings: ContainerSettings,
}

impl Container {
    pub fn load_from(base_path: impl Into<PathBuf>) -> Result<Self> {
        todo!();
        <Container as LocalResource<CoreContainer>>::rel_path();
        <Container as LocalResource<AssetMap>>::rel_path();
        <Container as LocalResource<ContainerSettings>>::rel_path();
    }

    /// Save all data.
    pub fn save(&mut self) -> Result {
        todo!();
        <Container as LocalResource<CoreContainer>>::path(self);
        <Container as LocalResource<AssetMap>>::path(self);
        <Container as LocalResource<ContainerSettings>>::path(self);
        Ok(())
    }

    /// Adds the given [`Asset`](LocalAsset) to the `Container`.
    pub fn insert_asset(&mut self, asset: Asset) -> Option<Asset> {
        self.container.assets.insert(asset.rid.clone(), asset)
    }

    /// Removes an [`Asset`](CoreAsset).
    /// Returns the removed `Asset` if it was present,
    /// or `None` otherwise.
    pub fn remove_asset(&mut self, rid: &ResourceId) -> Option<Asset> {
        self.assets.remove(rid)
    }

    // ---------------
    // --- scripts ---
    // ---------------

    /// Returns if the container is already associated with the script with the given id,
    /// regardless of the associations priority or autorun status.
    pub fn contains_script_association(&self, rid: &ResourceId) -> bool {
        self.scripts.get(rid).is_some()
    }

    /// Adds an association to the Container.
    /// Errors if an association with the script already exists.
    ///
    /// # See also
    /// + `set_script_association`
    pub fn add_script_association(&mut self, assoc: ScriptAssociation) -> Result {
        if self.contains_script_association(&assoc.script) {
            return Err(Error::CoreError(CoreError::ResourceError(
                ResourceError::AlreadyExists("Association with script already exists"),
            )));
        }

        let script = assoc.script.clone();
        self.scripts.insert(script, assoc.into());
        Ok(())
    }

    /// Sets or adds a script association with the Container.
    /// Returns whether or not the association with the script was added.
    ///
    /// # See also
    /// + `add_script_association`
    pub fn set_script_association(&mut self, assoc: ScriptAssociation) -> Result<bool> {
        let script = assoc.script.clone();
        let old = self.scripts.insert(script, assoc.into());
        Ok(old.is_none())
    }

    /// Removes as association with the given script.
    /// Returns if an association with the script existed.
    pub fn remove_script_association(&mut self, rid: &ResourceId) -> bool {
        let old = self.scripts.remove(rid);
        old.is_some()
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl PartialEq for Container {
    fn eq(&self, other: &Container) -> bool {
        self.container == other.container
    }
}

impl Eq for Container {}

impl Deref for Container {
    type Target = CoreContainer;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl DerefMut for Container {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}

impl Hash for Container {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rid.hash(state);
    }
}

impl HasId for Container {
    type Id = ResourceId;

    fn id(&self) -> &Self::Id {
        &self.container.id()
    }
}

impl LocalResource<CoreContainer> for Container {
    fn rel_path() -> PathBuf {
        container_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl LocalResource<AssetMap> for Container {
    fn rel_path() -> PathBuf {
        assets_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl LocalResource<ContainerSettings> for Container {
    fn rel_path() -> PathBuf {
        container_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
