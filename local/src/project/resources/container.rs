//! Container and container settings.
use crate::common::{assets_file, container_file, container_settings_file};
use crate::error::{Error, Result};
use crate::file_resource::LocalResource;
use crate::system::settings::UserSettings;
use crate::types::ContainerSettings;
use has_id::HasId;
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::container::AssetMap;
use thot_core::project::{
    container::ScriptMap, Container as CoreContainer,
    ContainerProperties as CoreContainerProperties, ScriptAssociation,
};
use thot_core::types::{Creator, ResourceId, UserId};

// **********************************
// *** Local Container Properties ***
// **********************************

pub struct ContainerProperties;
impl ContainerProperties {
    /// Creates a new [`ContainerProperties`](CoreContainerProperties) with fields actively filled from system settings.
    pub fn new(name: String) -> Result<CoreContainerProperties> {
        let settings = UserSettings::load()?;
        let creator = match settings.active_user.as_ref() {
            Some(uid) => Some(UserId::Id(uid.clone().into())),
            None => None,
        };

        let creator = Creator::User(creator);
        let mut props = CoreContainerProperties::new(name);
        props.creator = creator;

        Ok(props)
    }
}

// ***********************************
// *** Stored Container Properties ***
// ***********************************

/// Properties for a Container.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredContainerProperties {
    pub rid: ResourceId,
    pub properties: CoreContainerProperties,
    pub scripts: ScriptMap,
}

impl From<CoreContainer> for StoredContainerProperties {
    fn from(container: CoreContainer) -> Self {
        Self {
            rid: container.rid,
            properties: container.properties,
            scripts: container.scripts,
        }
    }
}

// *****************
// *** Container ***
// *****************

pub struct Container {
    base_path: PathBuf,
    container: CoreContainer,
    settings: ContainerSettings,
}

impl Container {
    /// Create a new Container at the given base path.
    ///
    /// # Notes
    /// + No changes or checks are made to the file system.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let name = PathBuf::from(path.clone());
        let name = name.file_name().expect("invalid path");
        let name: String = name.to_string_lossy().to_string();

        Self {
            base_path: path,
            container: CoreContainer::new(name),
            settings: ContainerSettings::default(),
        }
    }

    pub fn load_from(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();
        let properties_path =
            base_path.join(<Container as LocalResource<StoredContainerProperties>>::rel_path());

        let assets_path = base_path.join(<Container as LocalResource<AssetMap>>::rel_path());

        let settings_path =
            base_path.join(<Container as LocalResource<ContainerSettings>>::rel_path());

        let properties_file = fs::File::open(properties_path)?;
        let assets_file = fs::File::open(assets_path)?;
        let settings_file = fs::File::open(settings_path)?;

        let properties_reader = BufReader::new(properties_file);
        let assets_reader = BufReader::new(assets_file);
        let settings_reader = BufReader::new(settings_file);

        let container: StoredContainerProperties = serde_json::from_reader(properties_reader)?;
        let assets = serde_json::from_reader(assets_reader)?;
        let settings = serde_json::from_reader(settings_reader)?;

        let container = CoreContainer {
            rid: container.rid,
            properties: container.properties,
            assets,
            scripts: container.scripts,
        };

        Ok(Self {
            base_path,
            container,
            settings,
        })
    }

    /// Save all data.
    pub fn save(&self) -> Result {
        let properties_path = <Container as LocalResource<StoredContainerProperties>>::path(self);
        let assets_path = <Container as LocalResource<AssetMap>>::path(self);
        let settings_path = <Container as LocalResource<ContainerSettings>>::path(self);

        fs::create_dir_all(properties_path.parent().expect("invalid Container path"))?;
        let properties: StoredContainerProperties = self.container.clone().into();
        fs::write(properties_path, serde_json::to_string_pretty(&properties)?)?;
        fs::write(assets_path, serde_json::to_string_pretty(&self.assets)?)?;
        fs::write(settings_path, serde_json::to_string_pretty(&self.settings)?)?;

        Ok(())
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
        self.base_path = path.into();
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

impl LocalResource<StoredContainerProperties> for Container {
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
