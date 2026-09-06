//! App state types.
// NB: All mutation functions are implemented in `server::state`.
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, ops::Deref, path::PathBuf};
use syre_core::{
    project::{
        AnalysisAssociation, Asset as CoreAsset, Container as CoreContainer, ContainerProperties,
        Project as CoreProject,
    },
    types::ResourceId,
};
use syre_local::{
    error::IoSerde,
    project::{
        Flag,
        config::Settings,
        container::{Settings as ContainerSettings, StoredProperties as StoredContainerProperties},
    },
    types::AnalysisKind,
};

pub type ManifestState<T> = Result<Vec<T>, IoSerde>;
pub type ConfigState<T> = Result<T, IoSerde>;

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
    pub properties: DataResource<CoreProject>,
    pub settings: DataResource<Settings>,
    pub analyses: DataResource<Vec<Analysis>>,
}

impl ProjectData {
    pub fn properties(&self) -> DataResource<&CoreProject> {
        self.properties.as_ref().map_err(|err| err.clone())
    }

    pub fn settings(&self) -> DataResource<&Settings> {
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

    pub fn fs_resource(&self) -> &FileResource {
        &self.fs_resource
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
    #[serde(with = "crate::serde_os_string")]
    pub(crate) name: OsString,
    pub(crate) properties: DataResource<StoredContainerProperties>,
    pub(crate) settings: DataResource<ContainerSettings>,
    pub(crate) assets: DataResource<Vec<Asset>>,
    pub(crate) flags: DataResource<Vec<(PathBuf, Vec<Flag>)>>,
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
        self.settings.as_ref().map_err(|err| err.clone())
    }

    pub fn assets(&self) -> DataResource<&Vec<Asset>> {
        self.assets.as_ref().map_err(|err| err.clone())
    }

    pub fn analyses(&self) -> DataResource<&Vec<AnalysisAssociation>> {
        self.properties
            .as_ref()
            .map(|props| &props.analyses)
            .map_err(|err| err.clone())
    }

    pub fn flags(&self) -> DataResource<&Vec<(PathBuf, Vec<Flag>)>> {
        self.flags.as_ref().map_err(|err| err.clone())
    }

    /// Creates a `[syre_core::project::Container]`.
    ///
    /// # Returns
    /// `Some` if all states are valid, `None` otherwise.
    pub fn as_container(&self) -> Option<CoreContainer> {
        let DataResource::Ok(rid) = self.rid() else {
            return None;
        };
        let DataResource::Ok(properties) = self.properties() else {
            return None;
        };
        let DataResource::Ok(assets) = self.assets() else {
            return None;
        };
        let DataResource::Ok(analyses) = self.analyses() else {
            return None;
        };

        let assets = assets
            .into_iter()
            .map(|asset| asset.inner.clone())
            .collect();

        Some(CoreContainer::from_parts(
            rid.clone(),
            properties.clone(),
            assets,
            analyses.clone(),
        ))
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Asset {
    pub(crate) inner: CoreAsset,
    pub(crate) fs_resource: FileResource,
}

impl Asset {
    pub fn is_present(&self) -> bool {
        match self.fs_resource {
            FileResource::Present => true,
            FileResource::Absent => false,
        }
    }

    pub fn into_inner(self) -> CoreAsset {
        self.inner
    }
}

impl Deref for Asset {
    type Target = CoreAsset;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// # Notes
/// + Root node is at index 0.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Graph {
    pub nodes: Vec<Container>,

    /// Children of the node at the same index in `nodes`.
    /// Elements are indices of `nodes`.
    pub children: Vec<Vec<usize>>,
}

impl Graph {
    /// Construct all container paths.
    ///
    /// # Returns
    /// `Vec` of paths corresonding to `nodes`.
    pub fn paths(&self) -> Vec<PathBuf> {
        if self.nodes.is_empty() {
            return vec![];
        }

        let mut paths = vec![vec![]; self.nodes.len()];
        let mut pending = Vec::with_capacity(self.nodes.len());
        let mut idx = 0;
        paths[0].push(0);
        pending.push(0);
        while idx < pending.len() {
            let active = pending[idx];
            let children = &self.children[active];
            let root_path = paths[active].clone();
            for child_idx in children {
                let mut child_path = root_path.clone();
                child_path.push(*child_idx);
                paths[*child_idx] = child_path;
            }

            pending.append(&mut children.clone());
            idx += 1;
        }

        paths
            .into_iter()
            .map(|sequence| {
                sequence
                    .into_iter()
                    .map(|idx| &self.nodes[idx].name)
                    .collect::<PathBuf>()
            })
            .collect()
    }
}

pub type DataResource<T> = Result<T, IoSerde>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FolderResource<T> {
    Present(T),
    Absent,
}

impl<T> FolderResource<T> {
    pub fn is_present(&self) -> bool {
        matches!(self, FolderResource::Present(_))
    }
}

impl<T> FolderResource<T> {
    pub fn as_ref(&self) -> FolderResource<&T> {
        match *self {
            Self::Present(ref x) => FolderResource::Present(x),
            Self::Absent => FolderResource::Absent,
        }
    }

    #[track_caller]
    pub fn unwrap(self) -> T {
        if let Self::Present(x) = self {
            x
        } else {
            panic!("called `FolderResource::unwrap` on an `Absent` value");
        }
    }

    pub fn map<U, F>(&self, f: F) -> FolderResource<U>
    where
        F: FnOnce(&T) -> U,
    {
        match self {
            Self::Present(x) => FolderResource::Present(f(x)),
            Self::Absent => FolderResource::Absent,
        }
    }

    pub fn or_else<F>(self, f: F) -> FolderResource<T>
    where
        F: FnOnce() -> FolderResource<T>,
    {
        match self {
            x @ Self::Present(_) => x,
            Self::Absent => f(),
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum FileResource {
    Present,
    Absent,
}

impl FileResource {
    pub fn is_present(&self) -> bool {
        matches!(self, FileResource::Present)
    }
}

#[cfg(test)]
mod test {
    use super::{Container, Graph};
    use std::{ffi::OsString, io, path::PathBuf};
    use syre_local::error::IoSerde;

    #[test]
    fn graph_path() {
        let c0 = container_with_name("0");
        let c00 = container_with_name("0.0");
        let c01 = container_with_name("0.1");
        let c001 = container_with_name("0.0.1");

        let nodes = vec![c0, c00, c01, c001];
        let children = vec![vec![1, 2], vec![3], vec![], vec![]];
        let graph = Graph { nodes, children };

        let paths = graph.paths();
        assert_eq!(paths[0], PathBuf::from("0"));
        assert_eq!(paths[1], PathBuf::from("0/0.0"));
        assert_eq!(paths[2], PathBuf::from("0/0.1"));
        assert_eq!(paths[3], PathBuf::from("0/0.0/0.0.1"));
    }

    fn container_with_name(name: impl Into<OsString>) -> Container {
        Container {
            name: name.into(),
            properties: Err(IoSerde::Io(io::ErrorKind::NotFound)),
            settings: Err(IoSerde::Io(io::ErrorKind::NotFound)),
            assets: Err(IoSerde::Io(io::ErrorKind::NotFound)),
            flags: Err(IoSerde::Io(io::ErrorKind::NotFound)),
        }
    }
}
