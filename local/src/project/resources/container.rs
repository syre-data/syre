//! Container and container settings.
use crate::{
    common,
    error::{Error, IoSerde as IoSerdeError, Result},
    file_resource::LocalResource,
    system::settings::UserSettings,
    types::ContainerSettings,
};
use has_id::HasId;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    hash::{Hash, Hasher},
    io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    result::Result as StdResult,
};
use syre_core::{
    error::{Error as CoreError, Resource as ResourceError},
    project::{
        container::{AnalysisMap, AssetMap},
        AnalysisAssociation, Container as CoreContainer,
        ContainerProperties as CoreContainerProperties,
    },
    types::{Creator, ResourceId, UserId},
};

// ***********************************
// *** Stored Container Properties ***
// ***********************************

/// Properties for a Container.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredContainerProperties {
    pub rid: ResourceId,
    pub properties: CoreContainerProperties,
    pub analyses: AnalysisMap,
}

impl From<CoreContainer> for StoredContainerProperties {
    fn from(container: CoreContainer) -> Self {
        Self {
            rid: container.rid().clone(),
            properties: container.properties,
            analyses: container.analyses,
        }
    }
}

// *****************
// *** Container ***
// *****************

#[derive(Debug)]
pub struct Container {
    pub(crate) base_path: PathBuf,
    pub(crate) container: CoreContainer,
    pub(crate) settings: ContainerSettings,
}

impl Container {
    /// Create a new Container at the given base path.
    ///
    /// # Arguments
    /// 1. Path to the Container.
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

    /// Save all data.
    pub fn save(&self) -> StdResult<(), io::Error> {
        let properties_path = <Container as LocalResource<StoredContainerProperties>>::path(self);
        let assets_path = <Container as LocalResource<AssetMap>>::path(self);
        let settings_path = <Container as LocalResource<ContainerSettings>>::path(self);

        fs::create_dir_all(properties_path.parent().expect("invalid Container path"))?;
        let properties: StoredContainerProperties = self.container.clone().into();

        fs::write(
            properties_path,
            serde_json::to_string_pretty(&properties).unwrap(),
        )?;

        fs::write(
            assets_path,
            serde_json::to_string_pretty(&self.assets).unwrap(),
        )?;

        fs::write(
            settings_path,
            serde_json::to_string_pretty(&self.settings).unwrap(),
        )?;
        Ok(())
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
        self.base_path = path.into();
    }

    pub fn buckets(&self) -> Vec<PathBuf> {
        self.assets
            .values()
            .filter_map(|asset| asset.bucket())
            .collect()
    }

    // ---------------
    // --- analysis ---
    // ---------------

    /// Returns if the container is already associated with the analysis with the given id,
    /// regardless of the associations priority or autorun status.
    pub fn contains_analysis_association(&self, rid: &ResourceId) -> bool {
        self.analyses.get(rid).is_some()
    }

    /// Adds an association to the Container.
    /// Errors if an association with the analysis already exists.
    ///
    /// # See also
    /// + `set_analysis_association`
    pub fn add_analysis_association(&mut self, assoc: AnalysisAssociation) -> Result {
        if self.contains_analysis_association(assoc.analysis()) {
            return Err(Error::Core(CoreError::Resource(
                ResourceError::already_exists("Association with analysis already exists"),
            )));
        }

        let analysis = assoc.analysis().clone();
        self.analyses.insert(analysis, assoc.into());
        Ok(())
    }

    /// Sets or adds an analysis association with the Container.
    /// Returns whether or not the association with the analysis was added.
    ///
    /// # See also
    /// + [`add_analysis_association`]
    pub fn set_analysis_association(&mut self, assoc: AnalysisAssociation) -> bool {
        let analysis = assoc.analysis().clone();
        let old = self.analyses.insert(analysis, assoc.into());
        old.is_none()
    }

    /// Removes an association with the given analysis.
    /// Returns if an association with the analysis existed.
    pub fn remove_analysis_association(&mut self, rid: &ResourceId) -> bool {
        let old = self.analyses.remove(rid);
        old.is_some()
    }

    pub fn settings(&self) -> &ContainerSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ContainerSettings {
        &mut self.settings
    }

    /// Breaks self into parts.
    ///
    /// # Returns
    /// Tuple of (properties, settings, base path).
    pub fn into_parts(self) -> (CoreContainer, ContainerSettings, PathBuf) {
        let Self {
            container,
            base_path,
            settings,
        } = self;

        (container, settings, base_path)
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
        self.rid().hash(state);
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
        common::container_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl LocalResource<AssetMap> for Container {
    fn rel_path() -> PathBuf {
        common::assets_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl LocalResource<ContainerSettings> for Container {
    fn rel_path() -> PathBuf {
        common::container_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
