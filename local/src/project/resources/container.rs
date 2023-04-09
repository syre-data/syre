//! Container and container settings.
use super::asset::Assets;
use crate::common::{assets_file, container_file, container_settings_file};
use crate::error::{Error, Result};
use cluFlock::FlockLock;
use has_id::HasId;
use serde::{Deserialize, Serialize};
use settings_manager::error::Result as SettingsResult;
use settings_manager::local_settings::{
    Components as LocalComponents, Loader as LocalLoader, LocalSettings,
};
use settings_manager::settings::{self, Settings};
use settings_manager::types::Priority as SettingsPriority;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::container::{AssetMap, ScriptMap};
use thot_core::project::{
    Asset, Container as CoreContainer, ScriptAssociation, StandardProperties,
};
use thot_core::types::{ResourceId, UserPermissions};

// *****************
// *** Container ***
// *****************

pub struct Container {
    container_file_lock: FlockLock<File>,
    assets_file_lock: FlockLock<File>,
    settings_file_lock: FlockLock<File>,

    base_path: PathBuf,
    container: CoreContainer,
    settings: ContainerSettings,

    /// Used to store `ContainerProperties`.
    _properties: Box<ContainerProperties>,
}

impl Container {
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

    /// Save all data.
    pub fn save(&mut self) -> Result {
        <Container as Settings<ContainerProperties>>::save(self)?;
        <Container as Settings<AssetMap>>::save(self)?;
        <Container as Settings<ContainerSettings>>::save(self)?;
        Ok(())
    }
}

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

// --- Container Properties ---
impl Settings<ContainerProperties> for Container {
    fn settingss(&mut self) -> &ContainerProperties {
        *self._properties = self.container.clone().into();
        &*self._properties
    }

    fn file(&self) -> &File {
        &*self.container_file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut *self.container_file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.container_file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings<ContainerProperties> for Container {
    fn rel_path() -> PathBuf {
        container_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// --- Assets ---
impl Settings<AssetMap> for Container {
    fn settings(&self) -> &AssetMap {
        &self.assets
    }

    fn file(&self) -> &File {
        &*self.assets_file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut *self.assets_file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.assets_file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings<AssetMap> for Container {
    fn rel_path() -> PathBuf {
        assets_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// --- Container Settings ---
impl Settings<ContainerSettings> for Container {
    fn settings(&self) -> &ContainerSettings {
        &self.settings
    }

    fn file(&self) -> &File {
        &*self.settings_file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut *self.settings_file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.settings_file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings<ContainerSettings> for Container {
    fn rel_path() -> PathBuf {
        container_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl From<Loader> for Container {
    fn from(loader: Loader) -> Self {
        let loader: Components = loader.into();
        Self {
            container_file_lock: loader.container_file_lock,
            assets_file_lock: loader.assets_file_lock,
            settings_file_lock: loader.settings_file_lock,

            base_path: loader.base_path,
            container: loader.container.clone(),
            settings: loader.settings,
            _properties: Box::new(loader.container.into()),
        }
    }
}

// ****************************
// *** Container Properties ***
// ****************************

/// Container properties for persistance.
#[derive(Serialize, Deserialize)]
pub struct ContainerProperties {
    rid: ResourceId,
    properties: StandardProperties,
    scripts: ScriptMap,
}

impl ContainerProperties {
    pub fn scripts_mut(&mut self) -> &mut ScriptMap {
        &mut self.scripts
    }
}

impl Default for ContainerProperties {
    fn default() -> Self {
        Self {
            rid: ResourceId::new(),
            properties: StandardProperties::default(),
            scripts: ScriptMap::default(),
        }
    }
}

impl From<CoreContainer> for ContainerProperties {
    fn from(container: CoreContainer) -> Self {
        Self {
            rid: container.rid,
            properties: container.properties,
            scripts: container.scripts,
        }
    }
}

pub struct LocalContainerProperties {
    file_lock: FlockLock<File>,
    base_path: PathBuf,
    properties: ContainerProperties,
}

impl Deref for LocalContainerProperties {
    type Target = ContainerProperties;
    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}

impl DerefMut for LocalContainerProperties {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.properties
    }
}

impl Settings<ContainerProperties> for LocalContainerProperties {
    fn settings(&self) -> &ContainerProperties {
        &self.properties
    }

    fn file(&self) -> &File {
        &self.file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut self.file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings<ContainerProperties> for LocalContainerProperties {
    fn rel_path() -> PathBuf {
        container_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl From<LocalLoader<ContainerProperties>> for LocalContainerProperties {
    fn from(loader: LocalLoader<ContainerProperties>) -> Self {
        Self {
            file_lock: loader.file_lock(),
            base_path: loader.base_path(),
            properties: loader.data(),
        }
    }
}

// **************************
// *** Container Settings ***
// **************************

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ContainerSettings {
    permissions: Vec<UserPermissions>,
}

pub struct LocalContainerSettings {
    file_lock: FlockLock<File>,
    base_path: PathBuf,
    settings: ContainerSettings,
}

impl Settings<ContainerSettings> for LocalContainerSettings {
    fn settings(&self) -> &ContainerSettings {
        &self.settings
    }

    fn file(&self) -> &File {
        &self.file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut self.file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::Local
    }
}

impl LocalSettings<ContainerSettings> for LocalContainerSettings {
    fn rel_path() -> PathBuf {
        container_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// ***************
// *** Builder ***
// ***************

#[derive(Default)]
pub struct Builder {
    container: CoreContainer,
    settings: ContainerSettings,
}

impl Builder {
    pub fn container_mut(&mut self) -> &mut CoreContainer {
        &mut self.container
    }

    pub fn settings_mut(&mut self) -> &mut ContainerSettings {
        &mut self.settings
    }

    /// Convert to a [`Container`], creating files if needed.
    pub fn save(self, base_path: PathBuf) -> Result<Container> {
        settings::ensure_file(&base_path)?;
        let container_path = base_path.join(LocalContainerProperties::rel_path());
        let assets_path = base_path.join(Assets::rel_path());
        let settings_path = base_path.join(LocalContainerSettings::rel_path());

        let container_file = File::create(container_path)?;
        let assets_file = File::create(assets_path)?;
        let settings_file = File::create(settings_path)?;

        let container_file_lock = settings::lock(container_file)?;
        let assets_file_lock = settings::lock(assets_file)?;
        let settings_file_lock = settings::lock(settings_file)?;

        Ok(Container {
            container_file_lock,
            assets_file_lock,
            settings_file_lock,
            base_path,

            container: self.container.clone(),
            settings: self.settings,
            _properties: Box::new(self.container.into()),
        })
    }
}

// **************
// *** Loader ***
// **************

pub struct Loader {
    container_file_lock: FlockLock<File>,
    assets_file_lock: FlockLock<File>,
    settings_file_lock: FlockLock<File>,

    base_path: PathBuf,
    container: CoreContainer,
    settings: ContainerSettings,
}

impl Loader {
    pub fn load_or_create(base_path: PathBuf) -> SettingsResult<Self> {
        let properties_loader =
            LocalLoader::load_or_create::<LocalContainerProperties>(base_path.clone())?;
        let assets_loader = LocalLoader::load_or_create::<Assets>(base_path.clone())?;
        let settings_loader = LocalLoader::load_or_create::<LocalContainerSettings>(base_path)?;

        let properties_loader: LocalComponents<ContainerProperties> = properties_loader.into();
        let assets_loader: LocalComponents<AssetMap> = assets_loader.into();
        let settings_loader: LocalComponents<ContainerSettings> = settings_loader.into();

        let local_properties = properties_loader.data;
        let container = CoreContainer {
            rid: local_properties.rid,
            properties: local_properties.properties,
            scripts: local_properties.scripts,
            assets: assets_loader.data,
        };

        Ok(Self {
            container_file_lock: properties_loader.file_lock,
            assets_file_lock: assets_loader.file_lock,
            settings_file_lock: settings_loader.file_lock,

            base_path: properties_loader.base_path,
            container,
            settings: settings_loader.data,
        })
    }
}

impl Into<Components> for Loader {
    fn into(self) -> Components {
        Components {
            container_file_lock: self.container_file_lock,
            assets_file_lock: self.assets_file_lock,
            settings_file_lock: self.settings_file_lock,

            base_path: self.base_path,
            container: self.container,
            settings: self.settings,
        }
    }
}

pub struct Components {
    pub container_file_lock: FlockLock<File>,
    pub assets_file_lock: FlockLock<File>,
    pub settings_file_lock: FlockLock<File>,

    pub base_path: PathBuf,
    pub container: CoreContainer,
    pub settings: ContainerSettings,
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
