//! Container and container settings.
use super::asset::{Asset as LocalAsset, Assets as LocalAssets};
use super::standard_properties::StandardProperties;
use crate::common::{container_file_of, container_settings_file_of};
use crate::error::{Error, Result};
use crate::project::container::path_is_container;
use crate::types::{ResourceStore, ResourceValue};
use cluFlock::FlockLock;
use has_id::HasId;
use serde::{Deserialize, Serialize};
use settings_manager::error::{
    Error as SettingsError, Result as SettingsResult, SettingsError as LocalSettingsError,
};
use settings_manager::local_settings::{LocalSettings, LockSettingsFile};
use settings_manager::settings::{self, Settings};
use settings_manager::types::Priority as SettingsPriority;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thot_core::error::{ContainerError, Error as CoreError, ProjectError, ResourceError};
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
    pub parent: Option<ResourceId>,
    pub children: ContainerMap,

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
            parent: None,
            children: ContainerMap::new(),
            assets: LocalAssets::new(),
            scripts: ScriptMap::new(),
        })
    }

    /// Duplicates the tree.
    /// Copies the structure of the tree,
    /// along with `properties`, and `scripts`,
    /// into a new tree.
    ///
    /// # Note
    /// + Children should be loaded prior to call.
    /// See `load_children`.
    pub fn duplicate(&self) -> Result<Self> {
        let mut dup = Self::new()?;
        dup.properties = self.properties.clone();
        dup.scripts = self.scripts.clone();
        dup.parent = self.parent.clone();
        if let Ok(base_path) = self.base_path() {
            dup.set_base_path(base_path)?;
        }

        for child in self.children.clone().values() {
            let ResourceValue::Resource(child) = child else {
            // @todo: Return `Err`.
            panic!("child `Container` not loaded");
        };

            let child = child.lock().expect("could not lock child `Container`");
            let mut dup_child = child.duplicate()?;

            dup_child.parent = Some(dup.rid.clone());
            dup.children
                .insert_resource(dup_child.rid.clone(), dup_child);
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

    // ----------------
    // --- children ---
    // ----------------

    /// Registers a Container as a child.
    /// Returns `true` if the child was unregistered, i.e. newly added.
    ///
    /// # See also
    /// + `add_child`
    pub fn register_child(&mut self, rid: ResourceId) -> bool {
        if self.children.contains_key(&rid) {
            return false;
        }

        self.children.insert_id(rid);
        true
    }

    /// Deregisters a child.
    /// Returns `true` if the child was previously registered.
    pub fn deregister_child(&mut self, rid: &ResourceId) -> bool {
        if !self.children.contains_key(rid) {
            return false;
        }

        self.children.remove(rid);
        true
    }

    /// Gets the child with the given [`ResourceId`].
    ///
    /// # Agruments
    /// + `rid`: [`ResourceId`] of the child.
    ///
    /// # Returns
    /// The child `Container` if found, or `None`.
    ///
    /// # Errors
    /// + [`ProjectError::NotRegistered`]: If the given `ResourceId` is
    ///     not registered as a child.
    /// + [`ContainerError::MissingChild`]: If a child could not be found
    ///     with the given `ResourceId`.
    pub fn get_child(&self, rid: &ResourceId) -> Result<Container> {
        if !self.children.contains_key(rid) {
            return Err(Error::CoreError(CoreError::ProjectError(
                ProjectError::NotRegistered(Some(rid.clone()), None),
            )));
        }

        // iterate over children directories
        let r_path = self.base_path()?;
        for entry in fs::read_dir(&r_path)? {
            let entry = entry?;
            let e_type = entry.file_type()?;
            if e_type.is_dir() {
                // if child is a directory, check if it is a Container
                let p = entry.path();
                if !path_is_container(&p) {
                    continue;
                }

                let child = Self::load(&p)?;
                if &child.rid == rid {
                    return Ok(child);
                }
            }
        }

        // child not found in directories
        Err(Error::CoreError(CoreError::ContainerError(
            ContainerError::MissingChild(rid.clone()),
        )))
    }

    /// Gets the `Container`'s children without loading them into the `Container`s `children`.
    ///
    /// # Notes
    /// + Retrieves all `Containers` that are directly accessible,
    /// ignoring whether the child is registered or not.
    pub fn get_children(&self) -> Result<Vec<Container>> {
        let r_path = self.base_path()?;
        let mut children = Vec::new();

        // iterate over children directories
        for entry in fs::read_dir(&r_path)? {
            let entry = entry?;
            let e_type = entry.file_type()?;
            if e_type.is_dir() {
                // if child is a directory, check if it is a Container
                let p = entry.path();
                if !path_is_container(&p) {
                    continue;
                }

                let child = Self::load(&p)?;

                // add child path if path registered as child
                if self.children.contains_key(&child.rid) {
                    children.push(child);
                }
            }
        }

        Ok(children)
    }

    /// Loads the `Container`'s children.
    /// Overwrites any currently loaded value.
    ///
    /// # Arguments
    /// + `recurse`: Recurse down child tree.
    pub fn load_children(&mut self, recurse: bool) -> Result {
        let children = self.get_children()?;
        for mut child in children.into_iter() {
            if recurse {
                child.load_children(recurse)?
            }

            child.parent = Some(self.rid.clone());
            self.children.insert_resource(child.rid.clone(), child);
        }

        Ok(())
    }

    /// Sets the base path of children based on the root's base path.
    /// If a child's base path is unset, it remains unset.
    ///
    /// # Errors
    pub fn update_tree_base_paths(&mut self) -> Result {
        for child in self.children.clone().values() {
            let ResourceValue::Resource(child) = child else {
            // @todo: Return `Err`.
            panic!("child `Container` not loaded");
        };

            let mut child = child.lock().expect("could not lock child `Container`");
            let Ok(c_file_name) = child.base_path() else {
                continue;
            };

            let c_file_name = c_file_name
                .file_name()
                .expect("could not get file name of child `Container`");

            let mut c_path = self.base_path().expect("root base path not set");
            c_path.push(c_file_name);
            child
                .set_base_path(c_path)
                .expect("could not set base path");

            child.update_tree_base_paths()?;
        }

        Ok(())
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
            children: ContainerMap::new(),
            assets: LocalAssets::new(),
            scripts: ScriptMap::new(),
            parent: None,
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
            children: self.children.clone(),
            assets,
            scripts: self.scripts.clone(),
            parent: self.parent.clone(),
        }
    }
}

impl PartialEq for Container {
    fn eq(&self, other: &Self) -> bool {
        if !(self.rid == other.rid
            && self.properties == other.properties
            && self.parent == other.parent)
        {
            return false;
        }

        // children
        if self.children.len() != other.children.len() {
            return false;
        }

        if !self
            .children
            .keys()
            .all(|rid| other.children.contains_key(rid))
        {
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

        // children
        let mut children = ContainerMap::new();
        for (rid, child) in container.children.into_iter() {
            if let Some(child) = child {
                let child = child
                    .lock()
                    .expect("Container Mutex poisoned")
                    .clone()
                    .into();

                children.insert(
                    rid.clone(),
                    ResourceValue::Resource(Arc::new(Mutex::new(child))),
                );
            } else {
                children.insert_id(rid.clone());
            }
        }

        Container {
            _file_lock: None,
            _base_path: None,

            rid: container.rid,
            properties: container.properties,
            children,
            assets,
            scripts: container.scripts,
            parent: None,
        }
    }
}

impl Into<CoreContainer> for Container {
    /// Converts a `Container` into a [`thot_core::project::Container`].
    /// Clones and converts inner value of children, if a [`ResourceValue::Resource`].
    fn into(self) -> CoreContainer {
        let mut children = CoreContainerStore::new();
        for (rid, child) in self.children.into_iter() {
            if let ResourceValue::Resource(child) = child {
                let child: CoreContainer = child
                    .lock()
                    .expect("Container Mutex poisoned")
                    .clone()
                    .into();

                children.insert(rid.clone(), Some(Arc::new(Mutex::new(child))));
            } else {
                children.insert(rid.clone(), None);
            }
        }

        CoreContainer {
            rid: self.rid,
            properties: self.properties,
            parent: self.parent,
            children,
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
