//! Container and container settings.
use super::asset::{Asset as LocalAsset, Assets as LocalAssets};
use super::standard_properties::StandardProperties;
use crate::common::{container_file_of, container_settings_file_of};
use crate::error::{Error, Result};
use crate::types::ResourceStore;
use cluFlock::FlockLock;
use has_id::HasId;
use serde::{Deserialize, Serialize};
use settings_manager::error::{
    Error as SettingsError, Result as SettingsResult, SettingsError as LocalSettingsError,
};
use settings_manager::local_settings::{LocalSettings, LockSettingsFile};
use settings_manager::settings::{self, Settings};
use settings_manager::types::Priority as SettingsPriority;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::container::{ContainerStore as CoreContainerStore, ScriptMap};
use thot_core::project::{
    Asset as CoreAsset, Container as CoreContainer, ScriptAssociation,
    StandardProperties as CoreStandardProperties,
};
use thot_core::types::{ResourceId, ResourcePath, UserPermissions};

// *************
// *** Types ***
// *************

pub type ContainerMap = ResourceStore<Container>;
pub type ContainerWrapper = Arc<Mutex<Container>>;

// *****************
// *** Container ***
// *****************

#[derive(Serialize, Deserialize, HasId, Debug)]
pub struct Container {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _base_path: Option<PathBuf>,

    #[id]
    pub rid: ResourceId,
    pub properties: CoreStandardProperties,

    #[serde(skip)]
    pub assets: LocalAssets,

    #[serde(with = "serialize_scripts")]
    pub scripts: ScriptMap,
}

impl Container {
    // @todo: Change to Default.
    pub fn new() -> Result<Self> {
        let rid = ResourceId::new();
        let props = StandardProperties::new()?;

        Ok(Self {
            _file_lock: None,
            _base_path: None,

            rid,
            properties: props,
            assets: LocalAssets::new(),
            scripts: ScriptMap::new(),
        })
    }

    /// Duplicates the [`Container`].
    pub fn duplicate(&self) -> Result<Self> {
        let mut dup = Self::new()?;
        dup.properties = self.properties.clone();
        dup.scripts = self.scripts.clone();
        if let Ok(base_path) = self.base_path() {
            dup.set_base_path(base_path)?;
        }

        Ok(dup)
    }

    // --------------
    // --- Assets ---
    // --------------

    /// Initialize a new [`Asset`] and register it with the Container.
    /// Returns the [`ResourceId`] of the new [`Asset`].
    ///
    /// # Errors
    /// + [`LocalSettingsError::PathNotSet`]: If the base path of the container
    ///    is not set.
    pub fn new_asset(&mut self, path: &Path) -> Result<ResourceId> {
        let _cont_path = match self._base_path.clone() {
            Some(p) => p,
            None => return Err(SettingsError::SettingsError(LocalSettingsError::PathNotSet).into()),
        };

        // create asset
        let asset_path = ResourcePath::new(PathBuf::from(path))?;
        let asset = LocalAsset::new(asset_path)?;
        let rid = asset.rid.clone();

        self.insert_asset(asset)?;
        Ok(rid)
    }

    /// Adds the given [`Asset`](LocalAsset) to the `Container`.
    pub fn insert_asset(&mut self, asset: CoreAsset) -> Result {
        self.assets.insert_asset(asset)?;
        Ok(())
    }

    /// Removes an [`Asset`](CoreAsset).
    /// Returns the removed `Asset` if it was present,
    /// or `None` otherwise.
    pub fn remove_asset(&mut self, rid: &ResourceId) -> Option<CoreAsset> {
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
                ResourceError::AlreadyExists(String::from(
                    "Association with script already exists",
                )),
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
}

impl Default for Container {
    fn default() -> Self {
        Container {
            _file_lock: None,
            _base_path: None,

            rid: ResourceId::new(),
            properties: CoreStandardProperties::default(),
            assets: LocalAssets::new(),
            scripts: ScriptMap::new(),
        }
    }
}

impl Clone for Container {
    fn clone(&self) -> Self {
        let mut assets = LocalAssets::new();
        for (rid, asset) in self.assets.clone().into_iter() {
            assets.insert(rid, asset);
        }

        Container {
            _file_lock: None,
            _base_path: None,

            rid: self.rid.clone(),
            properties: self.properties.clone(),
            assets,
            scripts: self.scripts.clone(),
        }
    }
}

impl PartialEq for Container {
    fn eq(&self, other: &Self) -> bool {
        if !(self.rid == other.rid && self.properties == other.properties) {
            return false;
        }

        // assets
        if self.assets.len() != other.assets.len() {
            return false;
        }

        if !self.assets.keys().all(|rid| other.assets.contains_key(rid)) {
            return false;
        }

        // scripts
        if self.scripts.len() != other.scripts.len() {
            return false;
        }

        if !self
            .scripts
            .keys()
            .all(|rid| other.scripts.contains_key(rid))
        {
            return false;
        }

        // all equal
        true
    }
}

impl Eq for Container {}

// ****************
// *** settings ***
// ****************

impl Settings for Container {
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

impl LocalSettings for Container {
    fn rel_path() -> SettingsResult<PathBuf> {
        Ok(container_file_of(Path::new("")))
    }

    fn base_path(&self) -> SettingsResult<PathBuf> {
        self._base_path
            .clone()
            .ok_or(SettingsError::SettingsError(LocalSettingsError::PathNotSet))
    }

    // @todo: Should only be allowed to set if not loaded or unset?
    // Good case for builder or loader pattern?
    fn set_base_path(&mut self, path: PathBuf) -> SettingsResult {
        self._base_path = Some(path.clone());
        self.assets.set_base_path(path)?;
        Ok(())
    }

    fn load(base_path: &Path) -> SettingsResult<Self> {
        let rel_path = Self::rel_path()?;
        let path = base_path.join(rel_path);

        // load container
        let mut container = settings::load::<Self>(&path)?;
        container.set_base_path(base_path.to_path_buf())?;

        // load assets
        let assets = LocalAssets::load(base_path)?;
        container.assets = assets;

        Ok(container)
    }

    fn save(&mut self) -> SettingsResult {
        settings::save::<Self>(self)?;
        self.assets.save()?;
        Ok(())
    }
}

impl LockSettingsFile for Container {
    /// Acquire lock for self and `Asset`s.
    fn acquire_lock(&mut self) -> SettingsResult {
        // check lock is not already acquired
        if self.file().is_none() {
            let path = self.path()?;
            let file = settings::ensure_file(path.as_path())?;
            let file_lock = settings::lock(file)?;

            self.store_lock(file_lock);
        }

        self.assets.acquire_lock()?;
        Ok(())
    }
}

// **********************
// *** Core Container ***
// **********************

impl From<CoreContainer> for Container {
    /// Converts a [`thot_core::project::Container`] a into  `Container`.
    /// Clones and converts inner value of children, if a [`ResourceValue::Resource`].
    fn from(container: CoreContainer) -> Self {
        // assets
        let mut assets = LocalAssets::new();
        *assets = container.assets;

        Container {
            _file_lock: None,
            _base_path: None,

            rid: container.rid,
            properties: container.properties,
            assets,
            scripts: container.scripts,
        }
    }
}

impl Into<CoreContainer> for Container {
    /// Converts a `Container` into a [`thot_core::project::Container`].
    /// Clones and converts inner value of children, if a [`ResourceValue::Resource`].
    fn into(self) -> CoreContainer {
        let mut children = CoreContainerStore::new();

        CoreContainer {
            rid: self.rid,
            properties: self.properties,
            assets: (*self.assets).clone(),
            scripts: self.scripts,
        }
    }
}

// **********
// *** db ***
// **********

impl Hash for Container {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rid.hash(state);
    }
}

// *************
// *** serde ***
// *************

mod serialize_scripts {
    use super::ScriptMap;
    use serde::de;
    use serde::ser::{SerializeSeq, Serializer};
    use std::fmt;
    use std::result::Result as StdResult;
    use thot_core::project::ScriptAssociation;

    struct AssocVisitor;

    impl<'de> de::Visitor<'de> for AssocVisitor {
        type Value = ScriptMap;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a ScriptMap (HashMap<ResourceId, RunParameters>)")
        }

        fn visit_seq<A>(self, mut seq: A) -> StdResult<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut scripts: ScriptMap;
            match seq.size_hint() {
                None => scripts = ScriptMap::new(),
                Some(s) => scripts = ScriptMap::with_capacity(s),
            };

            while let Some(assoc) = seq.next_element::<ScriptAssociation>()? {
                scripts.insert(assoc.script.clone(), assoc.into());
            }

            Ok(scripts)
        }
    }

    /// Converts a Container's scripts in Script Association's for serialization.
    pub fn serialize<S>(scripts: &ScriptMap, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(scripts.len()))?;
        for (rid, params) in scripts.iter() {
            let assoc = params.clone().to_association(rid.clone());
            seq.serialize_element(&assoc)?;
        }

        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> StdResult<ScriptMap, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(AssocVisitor)
    }
}

// **************************
// *** Container Settings ***
// **************************

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ContainerSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _base_path: Option<PathBuf>,

    pub permissions: Vec<UserPermissions>,
}

impl ContainerSettings {
    pub fn new() -> Self {
        ContainerSettings {
            _file_lock: None,
            _base_path: None,

            permissions: Vec::new(),
        }
    }
}

impl Settings for ContainerSettings {
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

impl LocalSettings for ContainerSettings {
    fn rel_path() -> SettingsResult<PathBuf> {
        Ok(container_settings_file_of(Path::new("")))
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

impl LockSettingsFile for ContainerSettings {}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
