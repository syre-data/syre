use super::{
    fs,
    graph::{NodeMap, Tree},
    HasName, Ptr,
};
use has_id::HasId;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use syre_core::types::ResourceId;
use syre_local::common;

pub type ProjectMap = Vec<((Ptr<Project>, Ptr<Project>), ContainerMap)>;
pub type ContainerMap = NodeMap<Container>;

#[derive(Debug)]
pub struct State {
    app: AppState,
    projects: Vec<Ptr<Project>>,
}

impl State {
    pub fn new(user_manifest: impl Into<PathBuf>, project_manifest: impl Into<PathBuf>) -> Self {
        Self {
            app: AppState::new(user_manifest, project_manifest),
            projects: vec![],
        }
    }

    pub fn app_state(&self) -> &AppState {
        &self.app
    }

    pub fn projects(&self) -> &Vec<Ptr<Project>> {
        &self.projects
    }

    pub fn projects_mut(&mut self) -> &mut Vec<Ptr<Project>> {
        &mut self.projects
    }
}

impl State {
    /// Get the project associated to the path.
    pub fn find_path_project(&self, path: impl AsRef<Path>) -> Option<&Ptr<Project>> {
        let path = path.as_ref();
        self.projects
            .iter()
            .find(|project| path.starts_with(project.borrow().path()))
    }

    /// Get the project associated to the resource.
    pub fn find_resource_project(&self, resource: AppResource) -> Option<&Ptr<Project>> {
        match resource {
            AppResource::File(resource) => match resource {
                FileResource::UserManifest(manifest) => {
                    return None;
                }
                FileResource::ProjectManifest(_) => {
                    return None;
                }
                FileResource::ProjectProperties(properties) => {
                    self.projects
                        .iter()
                        .find(|project| match project.borrow().config() {
                            Resource::NotPresent => false,
                            Resource::Present(config) => {
                                Ptr::ptr_eq(config.borrow().properties(), &properties)
                            }
                        })
                }
                FileResource::ProjectSettings(settings) => {
                    self.projects
                        .iter()
                        .find(|project| match project.borrow().config() {
                            Resource::NotPresent => false,
                            Resource::Present(config) => {
                                Ptr::ptr_eq(config.borrow().settings(), &settings)
                            }
                        })
                }
                FileResource::AnalysisManifest(manifest) => {
                    self.projects
                        .iter()
                        .find(|project| match project.borrow().config() {
                            Resource::NotPresent => false,
                            Resource::Present(config) => {
                                Ptr::ptr_eq(config.borrow().analyses(), &manifest)
                            }
                        })
                }
                FileResource::Analysis(analysis) => self.projects.iter().find(|project| {
                    let project = project.borrow();
                    let Resource::Present(config) = project.config() else {
                        return false;
                    };

                    let config = config.borrow();
                    let analyses = config.analyses().borrow();
                    analyses
                        .manifest()
                        .iter()
                        .any(|a| Ptr::ptr_eq(&analysis, a))
                }),
                FileResource::ContainerProperties(properties) => {
                    self.projects.iter().find(|project| {
                        match project.borrow().data().borrow().graph() {
                            None => false,
                            Some(graph) => {
                                graph.nodes().iter().any(|node| match node.borrow().data() {
                                    None => false,
                                    Some(data) => Ptr::ptr_eq(
                                        data.config().borrow().properties(),
                                        &properties,
                                    ),
                                })
                            }
                        }
                    })
                }
                FileResource::ContainerSettings(settings) => self.projects.iter().find(|project| {
                    match project.borrow().data().borrow().graph() {
                        None => false,
                        Some(graph) => {
                            graph.nodes().iter().any(|node| match node.borrow().data() {
                                None => false,
                                Some(data) => {
                                    Ptr::ptr_eq(data.config().borrow().settings(), &settings)
                                }
                            })
                        }
                    }
                }),
                FileResource::AssetManifest(manifest) => self.projects.iter().find(|project| {
                    match project.borrow().data().borrow().graph() {
                        None => false,
                        Some(graph) => {
                            graph.nodes().iter().any(|node| match node.borrow().data() {
                                None => false,
                                Some(data) => {
                                    Ptr::ptr_eq(data.config().borrow().assets(), &manifest)
                                }
                            })
                        }
                    }
                }),
                FileResource::Asset(asset) => self.projects.iter().find(|project| {
                    match project.borrow().data().borrow().graph() {
                        None => false,
                        Some(graph) => {
                            graph.nodes().iter().any(|node| match node.borrow().data() {
                                None => false,
                                Some(data) => data
                                    .config()
                                    .borrow()
                                    .assets()
                                    .borrow()
                                    .manifest()
                                    .iter()
                                    .any(|a| Ptr::ptr_eq(a, &asset)),
                            })
                        }
                    }
                }),
            },

            AppResource::Folder(resource) => match resource {
                FolderResource::Project(project) => self
                    .projects()
                    .iter()
                    .find(|prj| Ptr::ptr_eq(prj, &project)),
                FolderResource::ProjectConfig(config) => {
                    self.projects
                        .iter()
                        .find(|project| match project.borrow().config() {
                            Resource::NotPresent => false,
                            Resource::Present(c) => Ptr::ptr_eq(c, &config),
                        })
                }
                FolderResource::Analyses(analyses) => self.projects.iter().find(|project| {
                    if let Some(a) = project.borrow().analyses() {
                        Ptr::ptr_eq(a, &analyses)
                    } else {
                        false
                    }
                }),
                FolderResource::ContainerTree(_) => None,
                FolderResource::Container(container) => self.projects.iter().find(|project| {
                    match project.borrow().data().borrow().graph() {
                        None => false,
                        Some(graph) => graph
                            .nodes()
                            .iter()
                            .any(|node| Ptr::ptr_eq(node, &container)),
                    }
                }),
                FolderResource::ContainerConfig(config) => self.projects.iter().find(|project| {
                    match project.borrow().data().borrow().graph() {
                        None => false,
                        Some(graph) => {
                            graph.nodes().iter().any(|node| match node.borrow().data() {
                                None => false,
                                Some(data) => Ptr::ptr_eq(data.config(), &config),
                            })
                        }
                    }
                }),
            },
        }
    }

    pub fn resource(&self, path: impl AsRef<Path>) -> Option<AppResource> {
        let path = path.as_ref();
        let user_manifest = &self.app.user_manifest;
        if path == user_manifest.borrow().path {
            return Some(FileResource::UserManifest(user_manifest.clone()).into());
        }

        let project_manifest = &self.app.project_manifest;
        if path == project_manifest.borrow().path {
            return Some(FileResource::ProjectManifest(project_manifest.clone()).into());
        }

        self.projects.iter().find_map(|project| {
            if path.starts_with(project.borrow().path()) {
                self.resource_project(path, project)
            } else {
                None
            }
        })
    }

    fn resource_project(
        &self,
        path: impl AsRef<Path>,
        project_ptr: &Ptr<Project>,
    ) -> Option<AppResource> {
        let path = path.as_ref();
        let project = project_ptr.borrow();

        if path == project.path() {
            return Some(FolderResource::Project(project_ptr.clone()).into());
        }

        let Ok(rel_path) = path.strip_prefix(project.path()) else {
            return None;
        };

        if rel_path == common::app_dir() {
            return Some(FolderResource::ProjectConfig(project.con)
        }
    }
}

impl State {
    /// Duplicate the state.
    /// All `fs` references point to the original resource.
    pub fn duplicate_with_fs_references_and_map(&self) -> (Self, ProjectMap) {
        let mut project_map = Vec::with_capacity(self.projects.len());
        let projects = self
            .projects
            .iter()
            .map(|project| {
                let (dup, container_map) = project.borrow().duplicate_with_fs_references_and_map();
                let dup = Ptr::new(dup);
                project_map.push(((project.clone(), dup.clone()), container_map));
                dup
            })
            .collect();

        (
            Self {
                app: self.app.clone(),
                projects,
            },
            project_map,
        )
    }
}

#[derive(Debug)]
pub struct AppState {
    user_manifest: Ptr<UserManifest>,
    project_manifest: Ptr<ProjectManifest>,
}

impl AppState {
    pub fn new(user_manifest: impl Into<PathBuf>, project_manifest: impl Into<PathBuf>) -> Self {
        Self {
            user_manifest: Ptr::new(UserManifest::new(user_manifest)),
            project_manifest: Ptr::new(ProjectManifest::new(project_manifest)),
        }
    }

    pub fn user_manifest(&self) -> &Ptr<UserManifest> {
        &self.user_manifest
    }

    pub fn project_manifest(&self) -> &Ptr<ProjectManifest> {
        &self.project_manifest
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        let user_manifest = Ptr::new(self.user_manifest.borrow().clone());
        let project_manifest = Ptr::new(self.project_manifest.borrow().clone());
        Self {
            user_manifest,
            project_manifest,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserManifest {
    path: PathBuf,
    manifest: Vec<String>,
}

impl UserManifest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            manifest: vec![],
        }
    }
}

impl HasPath for UserManifest {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl HasFsDataResource for UserManifest {
    type Resource = fs::File;
    fn fs_resource(&self, fs: &fs::State) -> FsDataResource<Self::Resource> {
        match fs.find_file(&self.path) {
            None => FsDataResource::NotPresent,
            Some(file) => {
                // TODO: Read file for validity.
                let state = DataResourceState::Valid;
                FsDataResource::Present {
                    resource: file,
                    state,
                }
            }
        }
    }
}

impl Manifest for UserManifest {
    type Item = String;
    fn manifest(&self) -> &Vec<Self::Item> {
        &self.manifest
    }

    fn push(&mut self, value: Self::Item) {
        self.manifest.push(value);
    }

    fn remove(&mut self, index: usize) -> Self::Item {
        self.manifest.swap_remove(index)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectManifest {
    path: PathBuf,
    manifest: Vec<PathBuf>,
}

impl ProjectManifest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            manifest: vec![],
        }
    }
}

impl HasPath for ProjectManifest {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl HasFsDataResource for ProjectManifest {
    type Resource = fs::File;
    fn fs_resource(&self, fs: &fs::State) -> FsDataResource<Self::Resource> {
        match fs.find_file(&self.path) {
            None => FsDataResource::NotPresent,
            Some(file) => {
                // TODO: Read file for validity.
                let state = DataResourceState::Valid;
                FsDataResource::Present {
                    resource: file,
                    state,
                }
            }
        }
    }
}

impl Manifest for ProjectManifest {
    type Item = PathBuf;
    fn manifest(&self) -> &Vec<Self::Item> {
        &self.manifest
    }

    fn push(&mut self, value: Self::Item) {
        self.manifest.push(value);
    }

    fn remove(&mut self, index: usize) -> Self::Item {
        self.manifest.swap_remove(index)
    }
}

#[derive(Debug, HasId)]
pub struct Project {
    #[id]
    rid: ResourceId,

    /// Path to the project's base folder.
    path: PathBuf,

    config: Resource<ProjectConfig>,

    /// Analyses folder.
    analyses: Option<Ptr<Analyses>>,

    /// Data folder.
    data: Ptr<Data>,
}

impl Project {
    pub fn new(path: impl Into<PathBuf>, data_path: impl Into<PathBuf>) -> Self {
        Self {
            rid: ResourceId::new(),
            path: path.into(),
            config: Resource::NotPresent,
            data: Ptr::new(Data::new(data_path)),
            analyses: None,
        }
    }

    pub fn rid(&self) -> &ResourceId {
        &self.rid
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.path = path.into();
    }

    pub fn config(&self) -> &Resource<ProjectConfig> {
        &self.config
    }

    pub fn remove_config(&mut self) {
        self.config = Resource::NotPresent;
    }

    pub fn analyses(&self) -> Option<&Ptr<Analyses>> {
        self.analyses.as_ref()
    }

    pub fn data(&self) -> &Ptr<Data> {
        &self.data
    }
}

impl Project {
    /// Duplicates the project.
    /// All `fs` references point to their original resource.
    ///
    /// # Returns
    /// Tuple of (duplicate, [(orginal container, duplicate container)]).
    pub fn duplicate_with_fs_references_and_map(&self) -> (Self, ContainerMap) {
        let config = if let Resource::Present(config) = &self.config {
            Resource::Present(Ptr::new(config.borrow().duplicate_with_fs_references()))
        } else {
            Resource::NotPresent
        };

        let (data, data_map) = self.data.borrow().duplicate_with_app_resource_and_map();
        let analyses = self
            .analyses
            .clone()
            .map(|analyses| Ptr::new(analyses.borrow().clone()));

        (
            Self {
                rid: self.rid.clone(),
                path: self.path.clone(),
                config,
                analyses,
                data: Ptr::new(data),
            },
            data_map,
        )
    }
}

impl HasFsResource for Project {
    type Resource = fs::Folder;
    fn fs_resource(&self, fs: &fs::State) -> FsResource<fs::Folder> {
        fs.find_folder(&self.path).into()
    }
}

#[derive(Debug)]
pub struct ProjectConfig {
    properties: Ptr<ProjectProperties>,
    settings: Ptr<ProjectSettings>,
    analyses: Ptr<AnalysisManifest>,
}

impl ProjectConfig {
    pub fn new() -> Self {
        Self {
            properties: Ptr::new(ProjectProperties),
            settings: Ptr::new(ProjectSettings),
            analyses: Ptr::new(AnalysisManifest::new()),
        }
    }

    pub fn properties(&self) -> &Ptr<ProjectProperties> {
        &self.properties
    }

    pub fn settings(&self) -> &Ptr<ProjectSettings> {
        &self.settings
    }

    pub fn analyses(&self) -> &Ptr<AnalysisManifest> {
        &self.analyses
    }
}

impl ProjectConfig {
    pub fn duplicate_with_fs_references(&self) -> Self {
        Self {
            properties: Ptr::new(self.properties.borrow().clone()),
            settings: Ptr::new(self.settings.borrow().clone()),
            analyses: Ptr::new(self.analyses.borrow().clone()),
        }
    }
}

impl HasFsResourceRelative for ProjectConfig {
    type Resource = fs::Folder;
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State) -> FsResource<Self::Resource> {
        fs.find_folder(common::app_dir_of(root)).into()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectProperties;
impl HasFsDataResourceRelative for ProjectProperties {
    type Resource = fs::File;
    fn fs_resource(
        &self,
        root: impl AsRef<Path>,
        fs: &fs::State,
    ) -> FsDataResource<Self::Resource> {
        let Some(file) = fs.find_file(common::project_file_of(root)) else {
            return FsDataResource::NotPresent;
        };

        // TODO: Read file and check validity.
        let state = DataResourceState::Valid;
        FsDataResource::Present {
            resource: file,
            state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectSettings;
impl HasFsDataResourceRelative for ProjectSettings {
    type Resource = fs::File;
    fn fs_resource(
        &self,
        root: impl AsRef<Path>,
        fs: &fs::State,
    ) -> FsDataResource<Self::Resource> {
        let Some(file) = fs.find_file(common::project_settings_file_of(root)) else {
            return FsDataResource::NotPresent;
        };

        // TODO: Read file and check validity.
        let state = DataResourceState::Valid;
        FsDataResource::Present {
            resource: file,
            state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisManifest {
    manifest: Vec<Ptr<Analysis>>,
}

impl AnalysisManifest {
    pub fn new() -> Self {
        Self { manifest: vec![] }
    }
}

impl HasFsDataResourceRelative for AnalysisManifest {
    type Resource = fs::File;
    fn fs_resource(
        &self,
        root: impl AsRef<Path>,
        fs: &fs::State,
    ) -> FsDataResource<Self::Resource> {
        let Some(file) = fs.find_file(common::analyses_file_of(root)) else {
            return FsDataResource::NotPresent;
        };

        // TODO: Read file and check validity.
        let state = DataResourceState::Valid;
        FsDataResource::Present {
            resource: file,
            state,
        }
    }
}

impl Manifest for AnalysisManifest {
    type Item = Ptr<Analysis>;
    fn manifest(&self) -> &Vec<Self::Item> {
        &self.manifest
    }

    fn push(&mut self, value: Self::Item) {
        self.manifest.push(value);
    }

    fn remove(&mut self, index: usize) -> Self::Item {
        self.manifest.swap_remove(index)
    }
}

/// Project analysis folder.
#[derive(Debug, Clone)]
pub struct Analyses {
    /// Path to the analysis folder.
    path: PathBuf,
}

impl Analyses {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl HasFsResourceRelative for Analyses {
    type Resource = fs::Folder;
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State) -> FsResource<Self::Resource> {
        let path = root.as_ref().join(&self.path);
        fs.find_folder(path).into()
    }
}

#[derive(Debug, HasId)]
pub struct Analysis {
    #[id]
    rid: ResourceId,
    path: PathBuf,
}

impl Analysis {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            rid: ResourceId::new(),
            path: path.into(),
        }
    }
}

impl HasFsResourceRelative for Analysis {
    type Resource = fs::File;
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State) -> FsResource<Self::Resource> {
        let path = root.as_ref().join(&self.path);
        fs.find_file(path).into()
    }
}

impl HasPath for Analysis {
    fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Project data folder.
#[derive(Debug)]
pub struct Data {
    /// Path to the data root.
    ///
    /// # Notes
    /// + Includes the data root's name,
    /// so must aware when `join`ing paths
    /// or it will be doubled.
    path: PathBuf,

    /// Data graph.
    /// `None` if a folder is not at the root path.
    graph: Option<Tree<Container>>,
}

impl Data {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            graph: None,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.path = path.into();
    }

    pub fn graph(&self) -> Option<&Tree<Container>> {
        self.graph.as_ref()
    }

    pub fn graph_mut(&mut self) -> Option<&mut Tree<Container>> {
        self.graph.as_mut()
    }

    /// Creates a new graph.
    ///
    /// # Panics
    /// + If a graph already exists.
    pub fn create_graph(&mut self) {
        assert!(self.graph.is_none());
        let root = Container::without_data(self.path.file_name().unwrap());
        let graph = Tree::new(root);
        self.graph = Some(graph);
    }

    pub fn remove_graph(&mut self) {
        self.graph = None;
    }

    /// Remove a container from the graph.
    ///
    /// # Panics
    /// + If `graph` is `None`.
    pub fn remove_container(&mut self, container: &Ptr<Container>) {
        let graph = self.graph.as_mut().unwrap();
        if Ptr::ptr_eq(&graph.root(), &container) {
            self.graph = None;
        } else {
            graph.remove(container).unwrap();
        }
    }
}

impl Data {
    /// Duplicates the the `Data` struct.
    /// The `project` reference and all app references in the `graph`
    /// point to the original resource.
    ///
    /// # Returns
    /// Tuple of (duplicate, container map) where `container map` is `[(original, duplicate)]`.
    /// `(None, [])` if `graph` is `None`.
    pub fn duplicate_with_app_resource_and_map(&self) -> (Self, ContainerMap) {
        let (graph, node_map) = if let Some(graph) = &self.graph {
            let (graph, node_map) = graph.duplicate_with_map();
            (Some(graph), node_map)
        } else {
            (None, vec![])
        };

        (
            Self {
                path: self.path.clone(),
                graph,
            },
            node_map,
        )
    }
}

/// Any folder in a Project's data directory.
/// It may or may not have Container data.
/// This is represented in the `data` field.
#[derive(Debug, Clone)]
pub struct Container {
    name: OsString,
    data: Option<ContainerData>,
}

impl Container {
    pub fn without_data(name: impl Into<OsString>) -> Self {
        Self {
            name: name.into(),
            data: None,
        }
    }

    pub fn rid(&self) -> Option<ResourceId> {
        if let Some(data) = &self.data {
            Some(data.rid().clone())
        } else {
            None
        }
    }

    pub fn data(&self) -> &Option<ContainerData> {
        &self.data
    }

    pub fn set_data(&mut self, data: ContainerData) {
        assert!(self.data.is_none());
        self.data.insert(data);
    }

    pub fn remove_data(&mut self) -> Option<ContainerData> {
        assert!(self.data.is_some());
        self.data.take()
    }
}

impl HasFsResourceRelative for Container {
    type Resource = fs::Folder;
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State) -> FsResource<Self::Resource> {
        let path = root.as_ref().join(&self.name);
        fs.find_folder(path).into()
    }
}

#[derive(Debug, HasId, Clone)]
pub struct ContainerData {
    #[id]
    rid: ResourceId,
    config: Ptr<ContainerConfig>,
}

impl ContainerData {
    pub fn new() -> Self {
        Self {
            rid: ResourceId::new(),
            config: Ptr::new(ContainerConfig::new()),
        }
    }

    pub fn rid(&self) -> &ResourceId {
        &self.rid
    }

    pub fn config(&self) -> &Ptr<ContainerConfig> {
        &self.config
    }
}

#[derive(Debug, Clone)]
pub struct ContainerConfig {
    properties: Ptr<ContainerProperties>,
    settings: Ptr<ContainerSettings>,
    assets: Ptr<AssetManifest>,
}

impl ContainerConfig {
    pub fn new() -> Self {
        Self {
            properties: Ptr::new(ContainerProperties),
            settings: Ptr::new(ContainerSettings),
            assets: Ptr::new(AssetManifest::new()),
        }
    }

    pub fn properties(&self) -> &Ptr<ContainerProperties> {
        &self.properties
    }

    pub fn settings(&self) -> &Ptr<ContainerSettings> {
        &self.settings
    }

    pub fn assets(&self) -> &Ptr<AssetManifest> {
        &self.assets
    }
}

impl HasFsResourceRelative for ContainerConfig {
    type Resource = fs::Folder;
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State) -> FsResource<Self::Resource> {
        fs.find_folder(common::app_dir_of(root)).into()
    }
}

#[derive(Debug)]
pub struct ContainerProperties;
impl HasFsDataResourceRelative for ContainerProperties {
    type Resource = fs::File;
    fn fs_resource(
        &self,
        root: impl AsRef<Path>,
        fs: &fs::State,
    ) -> FsDataResource<Self::Resource> {
        let Some(file) = fs.find_file(common::container_file_of(root)) else {
            return FsDataResource::NotPresent;
        };

        // TODO: Read file and check validity.
        let state = DataResourceState::Valid;
        FsDataResource::Present {
            resource: file,
            state,
        }
    }
}

#[derive(Debug)]
pub struct ContainerSettings;
impl HasFsDataResourceRelative for ContainerSettings {
    type Resource = fs::File;
    fn fs_resource(
        &self,
        root: impl AsRef<Path>,
        fs: &fs::State,
    ) -> FsDataResource<Self::Resource> {
        let Some(file) = fs.find_file(common::container_settings_file_of(root)) else {
            return FsDataResource::NotPresent;
        };

        // TODO: Read file and check validity.
        let state = DataResourceState::Valid;
        FsDataResource::Present {
            resource: file,
            state,
        }
    }
}

#[derive(Debug)]
pub struct AssetManifest {
    manifest: Vec<Ptr<Asset>>,
}

impl AssetManifest {
    pub fn new() -> Self {
        Self { manifest: vec![] }
    }
}

impl HasFsDataResourceRelative for AssetManifest {
    type Resource = fs::File;
    fn fs_resource(
        &self,
        root: impl AsRef<Path>,
        fs: &fs::State,
    ) -> FsDataResource<Self::Resource> {
        let Some(file) = fs.find_file(common::assets_file_of(root)) else {
            return FsDataResource::NotPresent;
        };

        // TODO: Read file and check validity.
        let state = DataResourceState::Valid;
        FsDataResource::Present {
            resource: file,
            state,
        }
    }
}

impl Manifest for AssetManifest {
    type Item = Ptr<Asset>;
    fn manifest(&self) -> &Vec<Self::Item> {
        &self.manifest
    }

    fn push(&mut self, value: Self::Item) {
        self.manifest.push(value);
    }

    fn remove(&mut self, index: usize) -> Self::Item {
        self.manifest.swap_remove(index)
    }
}

#[derive(Debug, HasId)]
pub struct Asset {
    #[id]
    rid: ResourceId,
    name: OsString,
}

impl Asset {
    pub fn new(name: impl Into<OsString>) -> Self {
        Self {
            rid: ResourceId::new(),
            name: name.into(),
        }
    }
}

impl HasFsResourceRelative for Asset {
    type Resource = fs::File;
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State) -> FsResource<Self::Resource> {
        fs.find_file(root.as_ref().join(&self.name)).into()
    }
}

#[derive(Debug)]
pub enum Resource<T> {
    NotPresent,
    Present(Ptr<T>),
}

impl<T> Resource<T> {
    pub fn is_present(&self) -> bool {
        match self {
            Self::Present { .. } => true,
            Self::NotPresent => false,
        }
    }
}

#[derive(Debug)]
pub enum DataResource<T> {
    NotPresent,
    Present {
        resource: Ptr<T>,
        state: DataResourceState,
    },
}

impl<T> DataResource<T> {
    pub fn is_present(&self) -> bool {
        match self {
            Self::Present { .. } => true,
            Self::NotPresent => false,
        }
    }
}

#[derive(Debug)]
pub enum FsResource<T> {
    NotPresent,
    Present(Ptr<T>),
}

impl<T> FsResource<T> {
    pub fn is_present(&self) -> bool {
        match self {
            Self::Present { .. } => true,
            Self::NotPresent => false,
        }
    }
}

impl<T> From<Option<Ptr<T>>> for FsResource<T> {
    fn from(value: Option<Ptr<T>>) -> Self {
        match value {
            None => Self::NotPresent,
            Some(val) => Self::Present(val),
        }
    }
}

#[derive(Debug)]
pub enum FsDataResource<T> {
    NotPresent,
    Present {
        resource: Ptr<T>,
        state: DataResourceState,
    },
}

impl<T> FsDataResource<T> {
    pub fn is_present(&self) -> bool {
        match self {
            Self::Present { .. } => true,
            Self::NotPresent => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataResourceState {
    Valid,
    Invalid,
}

#[derive(Clone)]
pub enum FileResource {
    UserManifest(Ptr<UserManifest>),
    ProjectManifest(Ptr<ProjectManifest>),
    ProjectProperties(Ptr<ProjectProperties>),
    ProjectSettings(Ptr<ProjectSettings>),
    AnalysisManifest(Ptr<AnalysisManifest>),
    Analysis(Ptr<Analysis>),
    ContainerProperties(Ptr<ContainerProperties>),
    ContainerSettings(Ptr<ContainerSettings>),
    AssetManifest(Ptr<AssetManifest>),
    Asset(Ptr<Asset>),
}

impl std::fmt::Debug for FileResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileResource::UserManifest(ptr) => {
                f.write_fmt(format_args!("UserManifest [{:?}]", ptr.as_ptr()))
            }
            FileResource::ProjectManifest(ptr) => {
                f.write_fmt(format_args!("ProjectManifest [{:?}]", ptr.as_ptr()))
            }
            FileResource::ProjectProperties(ptr) => {
                f.write_fmt(format_args!("ProjectProperties [{:?}]", ptr.as_ptr()))
            }
            FileResource::ProjectSettings(ptr) => {
                f.write_fmt(format_args!("ProjectSettings [{:?}]", ptr.as_ptr()))
            }
            FileResource::AnalysisManifest(ptr) => {
                f.write_fmt(format_args!("AnalysisManifest [{:?}]", ptr.as_ptr()))
            }
            FileResource::Analysis(ptr) => {
                f.write_fmt(format_args!("Analysis [{:?}]", ptr.as_ptr()))
            }
            FileResource::ContainerProperties(ptr) => {
                f.write_fmt(format_args!("ContainerProperties [{:?}]", ptr.as_ptr()))
            }
            FileResource::ContainerSettings(ptr) => {
                f.write_fmt(format_args!("ContainerSettings [{:?}]", ptr.as_ptr()))
            }
            FileResource::AssetManifest(ptr) => {
                f.write_fmt(format_args!("AssetManifest [{:?}]", ptr.as_ptr()))
            }
            FileResource::Asset(ptr) => f.write_fmt(format_args!("Asset [{:?}]", ptr.as_ptr())),
        }
    }
}

pub enum FolderResource {
    Project(Ptr<Project>),
    ProjectConfig(Ptr<ProjectConfig>),
    Analyses(Ptr<Analyses>),
    ContainerTree(Tree<Container>),
    Container(Ptr<Container>),
    ContainerConfig(Ptr<ContainerConfig>),
}

impl std::fmt::Debug for FolderResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FolderResource::Project(ptr) => {
                f.write_fmt(format_args!("Project [{:?}]", ptr.as_ptr()))
            }
            FolderResource::ProjectConfig(ptr) => {
                f.write_fmt(format_args!("ProjectConfig [{:?}]", ptr.as_ptr()))
            }
            FolderResource::Analyses(ptr) => {
                f.write_fmt(format_args!("Analyses [{:?}]", ptr.as_ptr()))
            }
            FolderResource::ContainerTree(tree) => {
                f.write_fmt(format_args!("ContainerTree {:?}", tree))
            }
            FolderResource::Container(ptr) => {
                f.write_fmt(format_args!("Container [{:?}]", ptr.as_ptr()))
            }
            FolderResource::ContainerConfig(ptr) => {
                f.write_fmt(format_args!("ContainerConfig [{:?}]", ptr.as_ptr()))
            }
        }
    }
}

#[derive(Debug, derive_more::From)]
pub enum AppResource {
    File(FileResource),
    Folder(FolderResource),
}

impl HasName for Container {
    fn name(&self) -> &std::ffi::OsStr {
        &self.name
    }

    fn set_name(&mut self, name: impl Into<std::ffi::OsString>) {
        self.name = name.into()
    }
}

impl HasName for Asset {
    fn name(&self) -> &std::ffi::OsStr {
        &self.name
    }

    fn set_name(&mut self, name: impl Into<std::ffi::OsString>) {
        self.name = name.into()
    }
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        match self {
            Self::NotPresent => Self::NotPresent,
            Self::Present(ptr) => Self::Present(ptr.clone()),
        }
    }
}

impl<T> Clone for DataResource<T> {
    fn clone(&self) -> Self {
        match self {
            Self::NotPresent => Self::NotPresent,
            Self::Present { resource, state } => Self::Present {
                resource: resource.clone(),
                state: state.clone(),
            },
        }
    }
}

impl<T> Clone for FsResource<T> {
    fn clone(&self) -> Self {
        match self {
            Self::NotPresent => Self::NotPresent,
            Self::Present(ptr) => Self::Present(ptr.clone()),
        }
    }
}

impl<T> Clone for FsDataResource<T> {
    fn clone(&self) -> Self {
        match self {
            Self::NotPresent => Self::NotPresent,
            Self::Present { resource, state } => Self::Present {
                resource: resource.clone(),
                state: state.clone(),
            },
        }
    }
}

pub trait HasPath {
    fn path(&self) -> &PathBuf;
}

pub trait HasFsResource {
    type Resource;
    fn fs_resource(&self, fs: &fs::State) -> FsResource<Self::Resource>;
}

pub trait HasFsDataResource {
    type Resource;
    fn fs_resource(&self, fs: &fs::State) -> FsDataResource<Self::Resource>;
}

pub trait HasFsResourceRelative {
    type Resource;

    /// Retrieve the associated file system resource.
    ///
    /// # Arguments
    /// #. `root`: The root path within the file system from which
    /// the resource should be searched.
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State) -> FsResource<Self::Resource>;
}

pub trait HasFsDataResourceRelative {
    type Resource;

    /// Retrieve the associated file system resource.
    ///
    /// # Arguments
    /// #. `root`: The root path within the file system from which
    /// the resource should be searched.
    fn fs_resource(&self, root: impl AsRef<Path>, fs: &fs::State)
        -> FsDataResource<Self::Resource>;
}

pub trait Manifest {
    type Item;
    fn manifest(&self) -> &Vec<Self::Item>;
    fn push(&mut self, value: Self::Item);
    fn remove(&mut self, index: usize) -> Self::Item;
}
