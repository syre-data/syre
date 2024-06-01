use super::{
    fs,
    graph::{NodeMap, Tree},
    HasName, Ptr, WPtr,
};
use has_id::HasId;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use syre_core::types::ResourceId;

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
    pub fn find_path_project(&self, path: impl AsRef<Path>) -> Option<&Ptr<Project>> {
        let path = path.as_ref();
        self.projects
            .iter()
            .find(|project| path.starts_with(project.borrow().path()))
    }

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
                    let Some(properties) = properties.upgrade() else {
                        return None;
                    };

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
                    let Some(settings) = settings.upgrade() else {
                        return None;
                    };

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
                    let Some(manifest) = manifest.upgrade() else {
                        return None;
                    };

                    self.projects
                        .iter()
                        .find(|project| match project.borrow().config() {
                            Resource::NotPresent => false,
                            Resource::Present(config) => {
                                Ptr::ptr_eq(config.borrow().analyses(), &manifest)
                            }
                        })
                }
                FileResource::Analysis(analysis) => {
                    let Some(analysis) = analysis.upgrade() else {
                        return None;
                    };

                    self.projects.iter().find(|project| {
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
                    })
                }
                FileResource::ContainerProperties(properties) => {
                    let Some(properties) = properties.upgrade() else {
                        return None;
                    };

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
                FileResource::ContainerSettings(settings) => {
                    let Some(settings) = settings.upgrade() else {
                        return None;
                    };

                    self.projects.iter().find(|project| {
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
                    })
                }
                FileResource::AssetManifest(manifest) => {
                    let Some(manifest) = manifest.upgrade() else {
                        return None;
                    };

                    self.projects.iter().find(|project| {
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
                    })
                }
                FileResource::Asset(asset) => {
                    let Some(asset) = asset.upgrade() else {
                        return None;
                    };

                    self.projects.iter().find(|project| {
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
                    })
                }
            },

            AppResource::Folder(resource) => match resource {
                FolderResource::Project(project) => {
                    let Some(project) = project.upgrade() else {
                        return None;
                    };

                    self.projects()
                        .iter()
                        .find(|prj| Ptr::ptr_eq(prj, &project))
                }
                FolderResource::ProjectConfig(config) => {
                    let Some(config) = config.upgrade() else {
                        return None;
                    };

                    self.projects
                        .iter()
                        .find(|project| match project.borrow().config() {
                            Resource::NotPresent => false,
                            Resource::Present(c) => Ptr::ptr_eq(c, &config),
                        })
                }
                FolderResource::Analyses(analyses) => {
                    let Some(analyses) = analyses.upgrade() else {
                        return None;
                    };

                    self.projects.iter().find(|project| {
                        if let Some(a) = project.borrow().analyses() {
                            Ptr::ptr_eq(a, &analyses)
                        } else {
                            false
                        }
                    })
                }
                FolderResource::Container(container) => {
                    let Some(container) = container.upgrade() else {
                        return None;
                    };

                    self.projects.iter().find(|project| {
                        match project.borrow().data().borrow().graph() {
                            None => false,
                            Some(graph) => graph
                                .nodes()
                                .iter()
                                .any(|node| Ptr::ptr_eq(node, &container)),
                        }
                    })
                }
                FolderResource::ContainerConfig(config) => {
                    let Some(config) = config.upgrade() else {
                        return None;
                    };

                    self.projects.iter().find(|project| {
                        match project.borrow().data().borrow().graph() {
                            None => false,
                            Some(graph) => {
                                graph.nodes().iter().any(|node| match node.borrow().data() {
                                    None => false,
                                    Some(data) => Ptr::ptr_eq(data.config(), &config),
                                })
                            }
                        }
                    })
                }
            },
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
    fs_resource: FsDataResource<fs::File>,
    manifest: Vec<String>,
}

impl UserManifest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fs_resource: FsDataResource::NotPresent,
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
    fn fs_resource(&self) -> &FsDataResource<Self::Resource> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<Self::Resource>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        }
    }

    /// Removes the file resource and clears the manifest.
    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
        self.manifest.clear();
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
    fs_resource: FsDataResource<fs::File>,
    manifest: Vec<PathBuf>,
}

impl ProjectManifest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fs_resource: FsDataResource::NotPresent,
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

    fn fs_resource(&self) -> &FsDataResource<fs::File> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<fs::File>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        }
    }

    /// Removes the file resource and clears the manifest.
    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
        self.manifest.clear();
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
    fs_resource: FsResource<fs::Folder>,

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
            fs_resource: FsResource::NotPresent,
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

    pub fn fs_resource(&self) -> &FsResource<fs::Folder> {
        &self.fs_resource
    }

    pub fn set_fs_resource(&mut self, fs_resource: &Ptr<fs::Folder>) {
        self.fs_resource = FsResource::Present(Ptr::downgrade(fs_resource));
    }

    pub fn config(&self) -> &Resource<ProjectConfig> {
        &self.config
    }

    pub fn set_config_folder(&mut self, folder: &Ptr<fs::Folder>) {
        assert!(!self.config.is_present());
        let config = ProjectConfig::new(Ptr::downgrade(folder));
        self.config = Resource::Present(Ptr::new(config));
    }

    pub fn remove_config(&mut self) {
        self.config = Resource::NotPresent;
    }

    pub fn analyses(&self) -> Option<&Ptr<Analyses>> {
        self.analyses.as_ref()
    }

    /// Sets the folder reference to the analyses folder.
    ///
    /// # Panics
    /// + If `analyses` is `None`.
    ///
    /// # Note
    /// + Must check `folder` is consistent with analyses path manually.
    pub fn set_analyses_folder_reference(&mut self, folder: &Ptr<fs::Folder>) {
        self.analyses
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_fs_resource(folder);
    }

    pub fn remove_analyses_folder_reference(&mut self) {
        self.analyses
            .as_ref()
            .unwrap()
            .borrow_mut()
            .remove_fs_resource();
    }

    pub fn data(&self) -> &Ptr<Data> {
        &self.data
    }

    pub fn set_data_root(&mut self, folder: &Ptr<fs::Folder>) {
        self.data.borrow_mut().set_graph_root(folder);
    }

    pub fn remove_data_root(&mut self) {
        self.data.borrow_mut().remove_graph();
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
                fs_resource: self.fs_resource.clone(),
                config,
                analyses,
                data: Ptr::new(data),
            },
            data_map,
        )
    }
}

#[derive(Debug)]
pub struct ProjectConfig {
    fs_resource: WPtr<fs::Folder>,

    properties: Ptr<ProjectProperties>,
    settings: Ptr<ProjectSettings>,
    analyses: Ptr<AnalysisManifest>,
}

impl ProjectConfig {
    pub fn new(fs_resource: WPtr<fs::Folder>) -> Self {
        Self {
            fs_resource,
            properties: Ptr::new(ProjectProperties::not_present()),
            settings: Ptr::new(ProjectSettings::not_present()),
            analyses: Ptr::new(AnalysisManifest::not_present()),
        }
    }

    pub fn fs_resource(&self) -> &WPtr<fs::Folder> {
        &self.fs_resource
    }

    pub fn set_fs_resource(&mut self, folder: &Ptr<fs::Folder>) {
        self.fs_resource = Ptr::downgrade(folder);
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
            fs_resource: self.fs_resource.clone(),
            properties: Ptr::new(self.properties.borrow().clone()),
            settings: Ptr::new(self.settings.borrow().clone()),
            analyses: Ptr::new(self.analyses.borrow().clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectProperties {
    fs_resource: FsDataResource<fs::File>,
}

impl ProjectProperties {
    pub fn valid(file: &Ptr<fs::File>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Valid,
            },
        }
    }

    pub fn invalid(file: &Ptr<fs::File>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Invalid,
            },
        }
    }

    pub fn not_present() -> Self {
        Self {
            fs_resource: FsDataResource::NotPresent,
        }
    }
}

impl HasFsDataResource for ProjectProperties {
    type Resource = fs::File;

    fn fs_resource(&self) -> &FsDataResource<Self::Resource> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<fs::File>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        }
    }

    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
    }
}

#[derive(Debug, Clone)]
pub struct ProjectSettings {
    fs_resource: FsDataResource<fs::File>,
}

impl ProjectSettings {
    pub fn valid(file: &Ptr<fs::File>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Valid,
            },
        }
    }

    pub fn invalid(file: &Ptr<fs::File>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Invalid,
            },
        }
    }

    pub fn not_present() -> Self {
        Self {
            fs_resource: FsDataResource::NotPresent,
        }
    }
}

impl HasFsDataResource for ProjectSettings {
    type Resource = fs::File;

    fn fs_resource(&self) -> &FsDataResource<Self::Resource> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<fs::File>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        }
    }

    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisManifest {
    fs_resource: FsDataResource<fs::File>,
    manifest: Vec<Ptr<Analysis>>,
}

impl AnalysisManifest {
    pub fn valid(file: &Ptr<fs::File>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Valid,
            },
            manifest: vec![],
        }
    }

    pub fn valid_with_manifest(file: &Ptr<fs::File>, manifest: Vec<Ptr<Analysis>>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Valid,
            },
            manifest,
        }
    }

    pub fn invalid(file: &Ptr<fs::File>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Invalid,
            },
            manifest: vec![],
        }
    }

    pub fn invalid_with_manifest(file: &Ptr<fs::File>, manifest: Vec<Ptr<Analysis>>) -> Self {
        Self {
            fs_resource: FsDataResource::Present {
                resource: Ptr::downgrade(file),
                state: DataResourceState::Invalid,
            },
            manifest,
        }
    }

    pub fn not_present() -> Self {
        Self {
            fs_resource: FsDataResource::NotPresent,
            manifest: vec![],
        }
    }
}

impl HasFsDataResource for AnalysisManifest {
    type Resource = fs::File;

    fn fs_resource(&self) -> &FsDataResource<Self::Resource> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<fs::File>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        }
    }

    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
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
    fs_resource: FsResource<fs::Folder>,
}

impl Analyses {
    pub fn not_present(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fs_resource: FsResource::NotPresent,
        }
    }

    /// # Notes
    /// + Must check `path` and `folder` are consistent manually.
    pub fn present(path: impl Into<PathBuf>, folder: &Ptr<fs::Folder>) -> Self {
        Self {
            path: path.into(),
            fs_resource: FsResource::Present(Ptr::downgrade(folder)),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn fs_resource(&self) -> &FsResource<fs::Folder> {
        &self.fs_resource
    }

    pub fn set_fs_resource(&mut self, folder: &Ptr<fs::Folder>) {
        self.fs_resource = FsResource::Present(Ptr::downgrade(folder));
    }

    pub fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsResource::NotPresent;
    }
}

#[derive(Debug, HasId)]
pub struct Analysis {
    #[id]
    rid: ResourceId,
    path: PathBuf,
    fs_resource: FsResource<fs::File>,
}

impl Analysis {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            rid: ResourceId::new(),
            path: path.into(),
            fs_resource: FsResource::NotPresent,
        }
    }

    pub fn fs_resource(&self) -> &FsResource<fs::File> {
        &self.fs_resource
    }

    pub fn set_fs_resource(&mut self, resource: &Ptr<fs::File>) {
        self.fs_resource = FsResource::Present(Ptr::downgrade(resource));
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

    pub fn graph(&self) -> &Option<Tree<Container>> {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut Option<Tree<Container>> {
        &mut self.graph
    }

    /// Sets the graph's root.
    /// Removes previous graph.
    pub fn set_graph_root(&mut self, folder: &Ptr<fs::Folder>) {
        let root = Container::without_data(folder);
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
    fs_resource: WPtr<fs::Folder>,
    data: Option<ContainerData>,
}

impl Container {
    pub fn without_data(folder: &Ptr<fs::Folder>) -> Self {
        Self {
            name: folder.borrow().name().to_os_string(),
            fs_resource: Ptr::downgrade(folder),
            data: None,
        }
    }

    pub fn fs_resource(&self) -> &WPtr<fs::Folder> {
        &self.fs_resource
    }

    pub fn set_fs_resource(&mut self, folder: &Ptr<fs::Folder>) {
        self.fs_resource = Ptr::downgrade(folder);
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

#[derive(Debug, HasId, Clone)]
pub struct ContainerData {
    #[id]
    rid: ResourceId,
    config: Ptr<ContainerConfig>,
}

impl ContainerData {
    pub fn new(folder: &Ptr<fs::Folder>) -> Self {
        Self {
            rid: ResourceId::new(),
            config: Ptr::new(ContainerConfig::new(folder)),
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
    fs_resource: WPtr<fs::Folder>,

    properties: Ptr<ContainerProperties>,
    settings: Ptr<ContainerSettings>,
    assets: Ptr<AssetManifest>,
}

impl ContainerConfig {
    pub fn new(folder: &Ptr<fs::Folder>) -> Self {
        Self {
            fs_resource: Ptr::downgrade(folder),
            properties: Ptr::new(ContainerProperties::not_present()),
            settings: Ptr::new(ContainerSettings::not_present()),
            assets: Ptr::new(AssetManifest::not_present()),
        }
    }

    pub fn fs_resource(&self) -> &WPtr<fs::Folder> {
        &self.fs_resource
    }

    pub fn set_fs_resource(&mut self, folder: &Ptr<fs::Folder>) {
        self.fs_resource = Ptr::downgrade(folder);
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

#[derive(Debug)]
pub struct ContainerProperties {
    fs_resource: FsDataResource<fs::File>,
}

impl ContainerProperties {
    pub fn not_present() -> Self {
        Self {
            fs_resource: FsDataResource::NotPresent,
        }
    }
}

impl HasFsDataResource for ContainerProperties {
    type Resource = fs::File;
    fn fs_resource(&self) -> &FsDataResource<fs::File> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<fs::File>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        };
    }

    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
    }
}

#[derive(Debug)]
pub struct ContainerSettings {
    fs_resource: FsDataResource<fs::File>,
}

impl ContainerSettings {
    pub fn not_present() -> Self {
        Self {
            fs_resource: FsDataResource::NotPresent,
        }
    }
}

impl HasFsDataResource for ContainerSettings {
    type Resource = fs::File;
    fn fs_resource(&self) -> &FsDataResource<fs::File> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<fs::File>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        };
    }

    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
    }
}

#[derive(Debug)]
pub struct AssetManifest {
    fs_resource: FsDataResource<fs::File>,
    manifest: Vec<Ptr<Asset>>,
}

impl AssetManifest {
    pub fn not_present() -> Self {
        Self {
            fs_resource: FsDataResource::NotPresent,
            manifest: vec![],
        }
    }
}

impl HasFsDataResource for AssetManifest {
    type Resource = fs::File;

    fn fs_resource(&self) -> &FsDataResource<Self::Resource> {
        &self.fs_resource
    }

    fn set_fs_resource(&mut self, file: &Ptr<fs::File>, state: DataResourceState) {
        self.fs_resource = FsDataResource::Present {
            resource: Ptr::downgrade(file),
            state,
        }
    }

    fn remove_fs_resource(&mut self) {
        assert!(self.fs_resource.is_present());
        self.fs_resource = FsDataResource::NotPresent;
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
    fs_resource: FsResource<fs::File>,
}

impl Asset {
    pub fn new(name: impl Into<OsString>) -> Self {
        Self {
            rid: ResourceId::new(),
            name: name.into(),
            fs_resource: FsResource::NotPresent,
        }
    }

    pub fn fs_resource(&self) -> &FsResource<fs::File> {
        &self.fs_resource
    }

    pub fn set_fs_resource(&mut self, resource: &Ptr<fs::File>) {
        self.fs_resource = FsResource::Present(Ptr::downgrade(resource));
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
    Present(WPtr<T>),
}

impl<T> FsResource<T> {
    pub fn is_present(&self) -> bool {
        match self {
            Self::Present { .. } => true,
            Self::NotPresent => false,
        }
    }
}

#[derive(Debug)]
pub enum FsDataResource<T> {
    NotPresent,
    Present {
        resource: WPtr<T>,
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
    UserManifest(WPtr<UserManifest>),
    ProjectManifest(WPtr<ProjectManifest>),
    ProjectProperties(WPtr<ProjectProperties>),
    ProjectSettings(WPtr<ProjectSettings>),
    AnalysisManifest(WPtr<AnalysisManifest>),
    Analysis(WPtr<Analysis>),
    ContainerProperties(WPtr<ContainerProperties>),
    ContainerSettings(WPtr<ContainerSettings>),
    AssetManifest(WPtr<AssetManifest>),
    Asset(WPtr<Asset>),
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

#[derive(Clone)]
pub enum FolderResource {
    Project(WPtr<Project>),
    ProjectConfig(WPtr<ProjectConfig>),
    Analyses(WPtr<Analyses>),
    Container(WPtr<Container>),
    ContainerConfig(WPtr<ContainerConfig>),
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
            FolderResource::Container(ptr) => {
                f.write_fmt(format_args!("Container [{:?}]", ptr.as_ptr()))
            }
            FolderResource::ContainerConfig(ptr) => {
                f.write_fmt(format_args!("ContainerConfig [{:?}]", ptr.as_ptr()))
            }
        }
    }
}

#[derive(Clone, Debug, derive_more::From)]
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

pub trait HasFsDataResource {
    type Resource;
    fn fs_resource(&self) -> &FsDataResource<Self::Resource>;
    fn set_fs_resource(&mut self, resource: &Ptr<Self::Resource>, state: DataResourceState);
    fn remove_fs_resource(&mut self);
}

pub trait Manifest {
    type Item;
    fn manifest(&self) -> &Vec<Self::Item>;
    fn push(&mut self, value: Self::Item);
    fn remove(&mut self, index: usize) -> Self::Item;
}
