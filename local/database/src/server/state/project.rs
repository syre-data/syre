//! Project state.
use super::Error;
use crate::state::{DataResource, FileResource, FolderResource};
pub use action::Action;
use std::path::PathBuf;
use syre_local::{file_resource::LocalResource, TryReducible};

/// Project state.
#[derive(Debug)]
pub struct State {
    path: PathBuf,
    fs_resource: FolderResource<project::State>,
}

impl State {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            fs_resource: FolderResource::Absent,
        }
    }

    pub fn with_project(path: impl Into<PathBuf>, project: project::State) -> Self {
        Self {
            path: path.into(),
            fs_resource: FolderResource::Present(project),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn fs_resource(&self) -> &FolderResource<project::State> {
        &self.fs_resource
    }
}

impl State {
    /// # Panics
    /// + If graph is present, but invalid.
    pub fn load(path: impl Into<PathBuf>) -> Self {
        use crate::state;
        use syre_local::project::resources::{project::LoadError, Analyses, Project};

        let mut state = Self::new(path);
        if !state.path().is_dir() {
            return state;
        }

        let mut project = project::Builder::default();
        match Project::load_from(state.path()) {
            Ok(prj) => {
                let (properties, settings, path) = prj.into_parts();
                assert_eq!(&path, state.path());

                project.set_properties_ok(properties);
                project.set_settings_ok(settings);
            }

            Err(LoadError {
                properties,
                settings,
            }) => {
                project.set_properties(properties);
                project.set_settings(settings);
            }
        };

        let analyses = Analyses::load_from(state.path()).map(|analyses| {
            let path = analyses.path();
            analyses
                .to_vec()
                .into_iter()
                .map(|analysis| match analysis {
                    syre_local::types::AnalysisKind::Script(ref script) => {
                        if path.join(&script.path).is_file() {
                            state::Analysis::present(analysis)
                        } else {
                            state::Analysis::absent(analysis)
                        }
                    }
                    syre_local::types::AnalysisKind::ExcelTemplate(ref template) => {
                        if path.join(&template.template.path).is_file() {
                            state::Analysis::present(analysis)
                        } else {
                            state::Analysis::absent(analysis)
                        }
                    }
                })
                .collect::<Vec<_>>()
        });
        project.set_analyses(analyses);

        if let Result::Ok(properties) = project.properties().as_ref() {
            if let Ok(graph) = graph::State::load(state.path().join(&properties.data_root)) {
                project.set_graph(graph)
            }
        }

        state
            .try_reduce(Action::CreateFolder(project.build()))
            .unwrap();

        state
    }
}

impl TryReducible for State {
    type Action = Action;
    type Error = Error;
    fn try_reduce(&mut self, action: Self::Action) -> Result<(), Self::Error> {
        match action {
            Action::SetPath(path) => {
                self.path = path;
                Ok(())
            }
            Action::RemoveFolder => {
                self.fs_resource = FolderResource::Absent;
                Ok(())
            }
            Action::CreateFolder(project) => {
                self.fs_resource = FolderResource::Present(project);
                Ok(())
            }
            Action::RemoveConfig
            | Action::SetProperties(_)
            | Action::SetSettings(_)
            | Action::SetAnalyses(_)
            | Action::SetAnalysesAbsent
            | Action::Graph(_)
            | Action::Container { .. } => {
                let FolderResource::Present(project) = self.fs_resource.as_mut() else {
                    return Err(Error::InvalidTransition);
                };

                project.try_reduce(action)
            }
        }
    }
}

pub mod project {
    use super::{action, graph, Action, DataResource, Error, FolderResource};
    use crate::state;
    use std::{
        io::{self, ErrorKind},
        path::PathBuf,
    };
    use syre_core::project::Project as CoreProject;
    use syre_local::{error::IoSerde, types::ProjectSettings, TryReducible};

    #[derive(Debug)]
    pub struct Builder {
        properties: DataResource<CoreProject>,
        settings: DataResource<ProjectSettings>,
        analyses: DataResource<Vec<state::Analysis>>,
        graph: FolderResource<graph::State>,
    }

    impl Builder {
        pub fn properties(&self) -> &DataResource<CoreProject> {
            &self.properties
        }

        pub fn set_properties(&mut self, properties: DataResource<CoreProject>) {
            self.properties = properties;
        }

        pub fn set_properties_ok(&mut self, properties: CoreProject) {
            self.properties = DataResource::Ok(properties);
        }

        pub fn set_properties_err(&mut self, properties: impl Into<IoSerde>) {
            self.properties = DataResource::Err(properties.into());
        }

        pub fn set_settings(&mut self, settings: DataResource<ProjectSettings>) {
            self.settings = settings;
        }

        pub fn set_settings_ok(&mut self, settings: ProjectSettings) {
            self.settings = DataResource::Ok(settings);
        }

        pub fn set_settings_err(&mut self, settings: impl Into<IoSerde>) {
            self.settings = DataResource::Err(settings.into());
        }

        pub fn set_analyses(&mut self, analyses: DataResource<Vec<state::Analysis>>) {
            self.analyses = analyses;
        }

        pub fn set_analyses_ok(&mut self, analyses: Vec<state::Analysis>) {
            self.analyses = DataResource::Ok(analyses);
        }

        pub fn set_analyses_err(&mut self, analyses: impl Into<IoSerde>) {
            self.analyses = DataResource::Err(analyses.into());
        }

        pub fn set_graph(&mut self, graph: graph::State) {
            self.graph = FolderResource::Present(graph);
        }

        pub fn remove_graph(&mut self) {
            self.graph = FolderResource::Absent;
        }

        pub fn build(self) -> State {
            let Self {
                properties,
                settings,
                analyses,
                graph,
            } = self;

            State {
                properties,
                settings,
                analyses,
                graph,
            }
        }
    }

    impl Default for Builder {
        /// Initialize all resources in a "missing" state.
        fn default() -> Self {
            Self {
                properties: DataResource::Err(ErrorKind::NotFound.into()),
                settings: DataResource::Err(ErrorKind::NotFound.into()),
                analyses: DataResource::Err(ErrorKind::NotFound.into()),
                graph: FolderResource::Absent,
            }
        }
    }

    #[derive(Debug)]
    pub struct State {
        properties: DataResource<CoreProject>,
        settings: DataResource<ProjectSettings>,
        analyses: DataResource<Vec<state::Analysis>>,
        graph: FolderResource<graph::State>,
    }

    impl State {
        pub fn properties(&self) -> DataResource<&CoreProject> {
            self.properties.as_ref().map_err(|err| err.clone())
        }

        pub fn settings(&self) -> DataResource<&ProjectSettings> {
            self.settings.as_ref().map_err(|err| err.clone())
        }

        pub fn analyses(&self) -> DataResource<&Vec<state::Analysis>> {
            self.analyses.as_ref().map_err(|err| err.clone())
        }

        pub fn graph(&self) -> &FolderResource<graph::State> {
            &self.graph
        }
    }

    impl TryReducible for State {
        type Action = Action;
        type Error = Error;
        fn try_reduce(&mut self, action: Self::Action) -> std::result::Result<(), Self::Error> {
            match action {
                Action::SetPath(_) | Action::RemoveFolder | Action::CreateFolder(_) => {
                    unreachable!("handled elsewhere");
                }

                Action::RemoveConfig => {
                    self.properties = DataResource::Err(io::ErrorKind::NotFound.into());
                    self.settings = DataResource::Err(io::ErrorKind::NotFound.into());
                    self.analyses = DataResource::Err(io::ErrorKind::NotFound.into());
                    Ok(())
                }
                Action::SetProperties(properties) => {
                    self.properties = properties;
                    Ok(())
                }
                Action::SetSettings(settings) => {
                    self.settings = settings;
                    Ok(())
                }
                Action::SetAnalyses(analyses) => {
                    self.analyses = analyses;
                    Ok(())
                }
                Action::SetAnalysesAbsent => {
                    if let Ok(analyses) = self.analyses.as_mut() {
                        for analysis in analyses.iter_mut() {
                            analysis.set_absent();
                        }
                    }

                    Ok(())
                }
                Action::Graph(action) => match action {
                    super::action::Graph::Set(graph) => {
                        self.graph = graph;
                        Ok(())
                    }
                },
                Action::Container { path, action } => self.try_reduce_container(path, action),
            }
        }
    }

    impl State {
        fn try_reduce_container(
            &mut self,
            path: PathBuf,
            action: action::Container,
        ) -> std::result::Result<(), Error> {
            let FolderResource::Present(graph) = &self.graph else {
                return Err(Error::DoesNotExist);
            };

            let Some(container) = graph.find(&path) else {
                return Err(Error::DoesNotExist);
            };

            let mut container = container.lock().unwrap();
            match action {
                action::Container::SetProperties(properties) => {
                    container.properties = properties;
                }
                action::Container::SetSettings(settings) => {
                    container.settings = settings;
                }
                action::Container::SetAssets(assets) => {
                    container.assets = assets;
                }
                action::Container::RemoveConfig => {
                    container.properties = DataResource::Err(IoSerde::Io(io::ErrorKind::NotFound));
                    container.settings = DataResource::Err(IoSerde::Io(io::ErrorKind::NotFound));
                    container.assets = DataResource::Err(IoSerde::Io(io::ErrorKind::NotFound));
                }
            }

            Ok(())
        }
    }
}

pub mod analysis {
    use super::FileResource;
    use crate::state::Analysis;
    use std::path::Path;
    use syre_local::types::AnalysisKind;

    impl Analysis {
        pub fn present(properties: AnalysisKind) -> Self {
            Self {
                properties,
                fs_resource: FileResource::Present,
            }
        }

        pub fn absent(properties: AnalysisKind) -> Self {
            Self {
                properties,
                fs_resource: FileResource::Absent,
            }
        }
    }

    impl Analysis {
        pub fn set_present(&mut self) {
            self.fs_resource = FileResource::Present;
        }

        pub fn set_absent(&mut self) {
            self.fs_resource = FileResource::Absent;
        }
    }

    /// Find an analysis by its path.
    ///
    /// # Arguments
    /// + `path`: Needle. Should be a relative path.
    /// + `analyses`: Haystack.
    pub fn find_analysis_by_path(
        path: impl AsRef<Path>,
        analyses: &Vec<Analysis>,
    ) -> Option<&Analysis> {
        let path = path.as_ref();
        analyses
            .iter()
            .find(|analysis| match analysis.properties() {
                AnalysisKind::Script(script) => path == script.path,
                AnalysisKind::ExcelTemplate(template) => path == template.template.path,
            })
    }

    /// Find an analysis by its path.
    ///
    /// # Arguments
    /// + `path`: Needle. Should be a relative path.
    /// + `analyses`: Haystack.
    pub fn find_analysis_by_path_mut(
        path: impl AsRef<Path>,
        analyses: &mut Vec<Analysis>,
    ) -> Option<&mut Analysis> {
        let path = path.as_ref();
        analyses
            .iter_mut()
            .find(|analysis| match analysis.properties() {
                AnalysisKind::Script(script) => path == script.path,
                AnalysisKind::ExcelTemplate(template) => path == template.template.path,
            })
    }
}

mod container {
    use super::{DataResource, FileResource};
    use crate::state::{Asset, Container};
    use serde::{Deserialize, Serialize};
    use std::{ffi::OsString, io, ops::Deref, path::Path};
    use syre_core::{
        project::{AnalysisAssociation, Asset as CoreAsset, ContainerProperties},
        types::ResourceId,
    };
    use syre_local::{
        loader::container::Loader, project::resources::container::StoredContainerProperties,
        types::ContainerSettings,
    };

    impl Container {
        /// # Errors
        /// + If the path is invalid.
        pub fn load(path: impl AsRef<Path>) -> Result<Self, io::ErrorKind> {
            let path = path.as_ref();
            let Some(name) = path.file_name() else {
                return Err(io::ErrorKind::InvalidFilename);
            };

            let syre_local::loader::container::State {
                properties,
                settings,
                assets,
            } = Loader::load_resources(path);

            let assets = assets.map(|assets| {
                assets
                    .into_iter()
                    .map(|asset| {
                        let fs_resource = match path.join(&asset.path).exists() {
                            true => FileResource::Present,
                            false => FileResource::Absent,
                        };

                        Asset {
                            properties: asset,
                            fs_resource,
                        }
                    })
                    .collect()
            });

            Ok(Self {
                name: name.to_os_string(),
                properties,
                settings,
                assets,
            })
        }
    }

    impl Asset {
        pub fn present(asset: CoreAsset) -> Self {
            Self {
                properties: asset,
                fs_resource: FileResource::Present,
            }
        }

        pub fn absent(asset: CoreAsset) -> Self {
            Self {
                properties: asset,
                fs_resource: FileResource::Absent,
            }
        }
    }
}

pub mod graph {
    use crate::state::{Container, Graph};
    use rayon::prelude::*;
    use std::{
        fs, io,
        path::Path,
        sync::{Arc, Mutex},
    };

    pub type Node = Arc<Mutex<Container>>;
    pub type EdgeMap = Vec<(Node, Vec<Node>)>;

    #[derive(Debug)]
    pub struct State {
        nodes: Vec<Node>,

        root: Node,

        /// Child-parent relations.
        parents: Vec<(Node, Node)>,

        /// Parent-children relations.
        children: EdgeMap,
    }

    impl State {
        pub fn new(root: Container) -> Self {
            let root = Arc::new(Mutex::new(root));
            Self {
                nodes: vec![root.clone()],
                root: root.clone(),
                parents: vec![],
                children: vec![(root, vec![])],
            }
        }

        /// # Errors
        /// + If `path` is not a directory.
        pub fn load(path: impl AsRef<Path>) -> Result<Self, io::ErrorKind> {
            let path = path.as_ref();
            if !path.exists() {
                return Err(io::ErrorKind::NotFound);
            }
            if !path.is_dir() {
                return Err(io::ErrorKind::NotADirectory);
            }

            Ok(rayon::ThreadPoolBuilder::new()
                .build()
                .unwrap()
                .install(move || Self::load_tree(path)))
        }

        /// Recursive loader.
        ///
        /// # Panics
        /// + If the path is invalid.
        fn load_tree(path: impl AsRef<Path>) -> Self {
            let path = path.as_ref();
            let root = Container::load(path).unwrap();
            let mut graph = Self::new(root);
            let children = fs::read_dir(path)
                .unwrap()
                .into_iter()
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|entry| Self::load_tree(entry.unwrap().path()))
                .collect::<Vec<_>>();

            let root = graph.root().clone();
            for child in children {
                graph.insert(&root, child);
            }

            graph
        }
    }

    impl State {
        pub fn root(&self) -> &Node {
            &self.root
        }

        pub fn insert(&mut self, parent: &Node, graph: Self) -> Result<(), error::Insert> {
            let Self {
                nodes,
                root,
                children,
                parents,
            } = graph;

            if self
                .nodes
                .iter()
                .find(|node| Arc::ptr_eq(node, parent))
                .is_none()
            {
                return Err(error::Insert::ParentNotFound);
            };

            let root_container = root.lock().unwrap();
            for child in self.children(parent).unwrap() {
                let container = child.lock().unwrap();
                if container.name() == root_container.name() {
                    return Err(error::Insert::NameCollision);
                }
            }
            drop(root_container);

            self.nodes.extend(nodes);
            self.children.extend(children);
            self.parents.extend(parents);
            self.parents.push((root.clone(), parent.clone()));
            self.children_mut(&parent).unwrap().push(root);
            Ok(())
        }

        /// Returns the children for the given node
        /// if the node exists in the graph.
        pub fn children(&self, parent: &Node) -> Option<&Vec<Node>> {
            self.children.iter().find_map(|(p, children)| {
                if Arc::ptr_eq(p, parent) {
                    Some(children)
                } else {
                    None
                }
            })
        }

        /// Returns the children for the given node
        /// if the node exists in the graph.
        fn children_mut(&mut self, parent: &Node) -> Option<&mut Vec<Node>> {
            self.children.iter_mut().find_map(|(p, children)| {
                if Arc::ptr_eq(p, parent) {
                    Some(children)
                } else {
                    None
                }
            })
        }

        /// Returns the given node's parent if the node exists
        /// in the graph and has a parent (i.e. Is not the root node).
        pub fn parent(&self, child: &Node) -> Option<&Node> {
            self.parents.iter().find_map(|(c, parent)| {
                if Arc::ptr_eq(c, child) {
                    Some(parent)
                } else {
                    None
                }
            })
        }

        /// Find a container py its path.
        ///
        /// # Arguments
        /// 1. `path`: Absolute path to container, with the
        /// project's data root being the root path.
        ///
        /// # Panics
        /// + If path is not absolute.
        /// + If any special path components are used.
        ///     This includes path prefixes, current dir, and parent dir.
        pub fn find(&self, path: impl AsRef<Path>) -> Option<&Node> {
            assert!(path.as_ref().is_absolute());
            let mut node = &self.root;
            for component in path.as_ref().components().skip(1) {
                match component {
                    std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
                    | std::path::Component::CurDir
                    | std::path::Component::ParentDir => {
                        panic!("invalid path");
                    }

                    std::path::Component::Normal(name) => {
                        let Some(child) = self.children(node).unwrap().iter().find(|child| {
                            let child = child.lock().unwrap();
                            child.name() == name
                        }) else {
                            return None;
                        };

                        node = child;
                    }
                }
            }

            Some(node)
        }
    }

    impl State {
        pub fn as_graph(&self) -> Graph {
            assert!(Arc::ptr_eq(&self.nodes[0], &self.root));

            let nodes = self
                .nodes
                .iter()
                .map(|node| {
                    let container = node.lock().unwrap();
                    (*container).clone()
                })
                .collect();

            let children = self
                .children
                .iter()
                .map(|(parent, children)| {
                    let parent_idx = self
                        .nodes
                        .iter()
                        .position(|node| Arc::ptr_eq(node, parent))
                        .unwrap();

                    let children_idx = children
                        .iter()
                        .map(|child| {
                            self.nodes
                                .iter()
                                .position(|node| Arc::ptr_eq(node, child))
                                .unwrap()
                        })
                        .collect();

                    (parent_idx, children_idx)
                })
                .collect();

            Graph { nodes, children }
        }
    }

    mod error {
        pub enum Insert {
            ParentNotFound,
            NameCollision,
        }
    }
}

pub(crate) mod action {
    use super::{graph, project::State as Project, DataResource, FolderResource};
    use crate::state;
    use std::path::PathBuf;
    use syre_core::project::Project as CoreProject;
    use syre_local::{
        project::resources::container::StoredContainerProperties,
        types::{ContainerSettings, ProjectSettings},
    };

    #[derive(Debug, derive_more::From)]
    pub enum Action {
        /// Sets the project's path.
        SetPath(PathBuf),

        /// Sets the project's base folder to be `Absent`.
        RemoveFolder,

        /// Sets the project's base folder to be `Present` with the given state.
        CreateFolder(Project),

        /// Sets all config resources to be absent.
        RemoveConfig,

        SetProperties(DataResource<CoreProject>),
        SetSettings(DataResource<ProjectSettings>),
        SetAnalyses(DataResource<Vec<state::Analysis>>),

        /// Sets all analyses' file system resource to be absent.
        /// Used if the project's analysis directory is removed.
        SetAnalysesAbsent,

        #[from]
        Graph(Graph),

        Container {
            /// Absolute path to the container.
            /// Root path indicates the graph root.
            path: PathBuf,
            action: Container,
        },
    }

    #[derive(Debug)]
    pub enum Graph {
        /// Sets the state of the graph.
        Set(FolderResource<graph::State>),
    }

    #[derive(Debug)]
    pub enum Container {
        SetProperties(DataResource<StoredContainerProperties>),
        SetSettings(DataResource<ContainerSettings>),
        SetAssets(DataResource<Vec<state::Asset>>),

        /// Sets all config resources to be absent.
        RemoveConfig,
    }
}

impl<T> FolderResource<T> {
    pub fn as_ref(&self) -> FolderResource<&T> {
        match *self {
            Self::Present(ref x) => FolderResource::Present(x),
            Self::Absent => FolderResource::Absent,
        }
    }

    pub fn as_mut(&mut self) -> FolderResource<&mut T> {
        match *self {
            Self::Present(ref mut x) => FolderResource::Present(x),
            Self::Absent => FolderResource::Absent,
        }
    }

    pub fn map<U, F>(&self, f: F) -> FolderResource<U>
    where
        F: FnOnce(&T) -> U,
    {
        match self {
            Self::Present(ref x) => FolderResource::Present(f(x)),
            Self::Absent => FolderResource::Absent,
        }
    }

    pub fn is_present(&self) -> bool {
        match self {
            Self::Absent => false,
            Self::Present(_) => true,
        }
    }
}
