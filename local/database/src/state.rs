//! App state types.
// NB: All mutation functions are implemented in `server::state`.
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, ops::Deref, path::PathBuf};
use syre_core::{
    project::{
        AnalysisAssociation, Asset as CoreAsset, ContainerProperties, Project as CoreProject,
    },
    types::ResourceId,
};
use syre_local::{
    error::IoSerde,
    project::resources::container::StoredContainerProperties,
    types::{AnalysisKind, ContainerSettings, ProjectSettings},
};

pub type ManifestState<T> = Result<Vec<T>, IoSerde>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub(crate) path: PathBuf,
    pub(crate) fs_resource: FolderResource<ProjectData>,
}

impl Project {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn fs_resource(&self) -> &FolderResource<ProjectData> {
        &self.fs_resource
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectData {
    pub(crate) properties: DataResource<CoreProject>,
    pub(crate) settings: DataResource<ProjectSettings>,
    pub(crate) analyses: DataResource<Vec<Analysis>>,
}

impl ProjectData {
    pub fn properties(&self) -> DataResource<&CoreProject> {
        self.properties.as_ref().map_err(|err| err.clone())
    }

    pub fn settings(&self) -> DataResource<&ProjectSettings> {
        self.settings.as_ref().map_err(|err| err.clone())
    }

    pub fn analyses(&self) -> DataResource<&Vec<Analysis>> {
        self.analyses.as_ref().map_err(|err| err.clone())
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Analysis {
    pub(crate) properties: AnalysisKind,
    pub(crate) fs_resource: FileResource,
}

impl Analysis {
    pub fn properties(&self) -> &AnalysisKind {
        &self.properties
    }

    pub fn is_present(&self) -> bool {
        matches!(self.fs_resource, FileResource::Present)
    }
}

impl Deref for Analysis {
    type Target = AnalysisKind;
    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Container {
    /// Name of the container's folder.
    pub(crate) name: OsString,
    pub(crate) properties: DataResource<StoredContainerProperties>,
    pub(crate) settings: DataResource<ContainerSettings>,
    pub(crate) assets: DataResource<Vec<Asset>>,
}

impl Container {
    pub fn name(&self) -> &OsString {
        &self.name
    }

    pub fn rid(&self) -> DataResource<&ResourceId> {
        self.properties
            .as_ref()
            .map(|props| &props.rid)
            .map_err(|err| err.clone())
    }

    pub fn properties(&self) -> DataResource<&ContainerProperties> {
        self.properties
            .as_ref()
            .map(|props| &props.properties)
            .map_err(|err| err.clone())
    }

    pub fn settings(&self) -> DataResource<&ContainerSettings> {
        self.settings
            .as_ref()
            .map(|settings| settings)
            .map_err(|err| err.clone())
    }

    pub fn assets(&self) -> DataResource<&Vec<Asset>> {
        self.assets
            .as_ref()
            .map(|assets| assets)
            .map_err(|err| err.clone())
    }

    pub fn analyses(&self) -> DataResource<&Vec<AnalysisAssociation>> {
        self.properties
            .as_ref()
            .map(|props| &props.analyses)
            .map_err(|err| err.clone())
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Asset {
    pub(crate) properties: CoreAsset,
    pub(crate) fs_resource: FileResource,
}

impl Asset {
    pub fn is_present(&self) -> bool {
        match self.fs_resource {
            FileResource::Present => true,
            FileResource::Absent => false,
        }
    }
}

impl Deref for Asset {
    type Target = CoreAsset;
    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}

/// # Notes
/// + Root node is at index 0.
#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    pub nodes: Vec<Container>,

    /// Parent-children relations.
    pub children: Vec<(usize, Vec<usize>)>,
}

pub type DataResource<T> = Result<T, IoSerde>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FolderResource<T> {
    Present(T),
    Absent,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum FileResource {
    Present,
    Absent,
}
