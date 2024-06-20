//! Project state.
use crate::server::state;

use super::Error;
pub use action::Action;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_local::{error::IoSerde, file_resource::LocalResource, TryReducible};

/// Project state.
#[derive(Serialize, Deserialize, Debug)]
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
    pub fn load_from(path: impl Into<PathBuf>) -> Self {
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
                                state::project::analysis::State::present(analysis)
                            } else {
                                state::project::analysis::State::absent(analysis)
                            }
                        }
                        syre_local::types::AnalysisKind::ExcelTemplate(ref template) => {
                            if path.join(&template.template.path).is_file() {
                                state::project::analysis::State::present(analysis)
                            } else {
                                state::project::analysis::State::absent(analysis)
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            });

            project.set_analyses(analyses);

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
            | Action::SetAnalysesAbsent => {
                let FolderResource::Present(project) = self.fs_resource.as_mut() else {
                    return Err(Error::InvalidTransition);
                };

                project.try_reduce(action)
            }
        }
    }
}

pub mod project {
    use super::{analysis, graph, Action, DataResource, Error, FolderResource};
    use serde::{Deserialize, Serialize};
    use std::io::{self, ErrorKind};
    use syre_core::project::Project as CoreProject;
    use syre_local::{error::IoSerde, types::ProjectSettings, TryReducible};

    #[derive(Debug)]
    pub struct Builder {
        properties: DataResource<CoreProject>,
        settings: DataResource<ProjectSettings>,
        analyses: DataResource<Vec<analysis::State>>,
        graph: FolderResource<graph::State>,
    }

    impl Builder {
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

        pub fn set_analyses(&mut self, analyses: DataResource<Vec<analysis::State>>) {
            self.analyses = analyses;
        }

        pub fn set_analyses_ok(&mut self, analyses: Vec<analysis::State>) {
            self.analyses = DataResource::Ok(analyses);
        }

        pub fn set_analyses_err(&mut self, analyses: impl Into<IoSerde>) {
            self.analyses = DataResource::Err(analyses.into());
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

    #[derive(Serialize, Deserialize, Debug)]
    pub struct State {
        properties: DataResource<CoreProject>,
        settings: DataResource<ProjectSettings>,
        analyses: DataResource<Vec<analysis::State>>,
        graph: FolderResource<()>,
    }

    impl State {
        pub fn properties(&self) -> &DataResource<CoreProject> {
            &self.properties
        }

        pub fn settings(&self) -> &DataResource<ProjectSettings> {
            &self.settings
        }

        pub fn analyses(&self) -> &DataResource<Vec<analysis::State>> {
            &self.analyses
        }

        pub fn graph(&self) -> &FolderResource<()> {
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
            }
        }
    }
}

pub mod analysis {
    use super::FileResource;
    use serde::{Deserialize, Serialize};
    use std::{
        ops::Deref,
        path::{Path, PathBuf},
    };
    use syre_local::{
        file_resource::LocalResource, project::resources::Analyses, types::AnalysisKind,
    };

    #[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
    pub struct State {
        properties: AnalysisKind,
        fs_resource: FileResource,
    }

    impl State {
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

        /// Create from list of analyses by if checking paths are present in the file system.
        ///
        /// # Arguments
        /// + `path`: Path to the analysis root.
        /// + `analyses`: List of analysis properties.
        pub fn from_analyses(analyses: Analyses) -> Vec<Self> {
            Self::from_resources(analyses.path(), analyses.to_vec())
        }

        /// Create from list of analyses by if checking paths are present in the file system.
        ///
        /// # Arguments
        /// + `path`: Path to the analysis root.
        /// + `analyses`: List of analysis properties.
        pub fn from_resources(path: impl Into<PathBuf>, analyses: Vec<AnalysisKind>) -> Vec<Self> {
            let path = path.into();
            analyses
                .into_iter()
                .map(|analysis| match analysis {
                    syre_local::types::AnalysisKind::Script(ref script) => {
                        if path.join(&script.path).is_file() {
                            Self::present(analysis)
                        } else {
                            Self::absent(analysis)
                        }
                    }
                    syre_local::types::AnalysisKind::ExcelTemplate(ref template) => {
                        if path.join(&template.template.path).is_file() {
                            Self::present(analysis)
                        } else {
                            Self::absent(analysis)
                        }
                    }
                })
                .collect()
        }
    }

    impl State {
        pub fn properties(&self) -> &AnalysisKind {
            &self.properties
        }

        pub fn is_present(&self) -> bool {
            matches!(self.fs_resource, FileResource::Present)
        }

        pub fn set_present(&mut self) {
            self.fs_resource = FileResource::Present;
        }

        pub fn set_absent(&mut self) {
            self.fs_resource = FileResource::Absent;
        }
    }

    impl Deref for State {
        type Target = AnalysisKind;
        fn deref(&self) -> &Self::Target {
            &self.properties
        }
    }

    /// Find an analysis by its path.
    ///
    /// # Arguments
    /// + `path`: Needle. Should be af relative path.
    /// + `analyses`: Haystack.
    pub fn find_analysis_by_path(path: impl AsRef<Path>, analyses: &Vec<State>) -> Option<&State> {
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
    /// + `path`: Needle. Should be af relative path.
    /// + `analyses`: Haystack.
    pub fn find_analysis_by_path_mut(
        path: impl AsRef<Path>,
        analyses: &mut Vec<State>,
    ) -> Option<&mut State> {
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
    use std::{ffi::OsString, ops::Deref, path::Path};
    use syre_core::project::{container::AnalysisMap, Asset as CoreAsset, ContainerProperties};
    use syre_local::types::ContainerSettings;

    #[derive(Debug)]
    pub struct State {
        /// Name of the container's folder.
        name: OsString,
        properties: DataResource<ContainerProperties>,
        settings: DataResource<ContainerSettings>,
        assets: DataResource<Vec<Asset>>,
        analyses: DataResource<AnalysisMap>,
    }

    impl State {
        pub fn load_from(path: impl AsRef<Path>)-> Self {
            let path = path.as_ref();

            let name = 
        }

        pub fn name(&self) -> &OsString {
            &self.name
        }

        pub fn properties(&self) -> &DataResource<ContainerProperties> {
            &self.properties
        }

        pub fn settings(&self) -> &DataResource<ContainerSettings> {
            &self.settings
        }

        pub fn assets(&self) -> &DataResource<Vec<Asset>> {
            &self.assets
        }

        pub fn analyses(&self) -> &DataResource<AnalysisMap> {
            &self.analyses
        }
    }

    #[derive(Debug)]
    pub struct Asset {
        properties: CoreAsset,
        fs_resource: FileResource,
    }

    impl Deref for Asset {
        type Target = CoreAsset;
        fn deref(&self) -> &Self::Target {
            &self.properties
        }
    }
}

mod graph {
    use super::container::State as Container;
    use std::{
        cell::RefCell,
        rc::{Rc, Weak},
    };

    pub type Node = Rc<RefCell<Container>>;
    pub type NodeRef = Weak<RefCell<Container>>;

    #[derive(Debug)]
    pub struct State {
        nodes: Vec<Node>,

        root: NodeRef,

        /// child-parent relationships.
        parents: Vec<(NodeRef, NodeRef)>,

        /// parent-children relationships.
        children: Vec<(NodeRef, Vec<NodeRef>)>,
    }

    impl State {
        pub fn new(root: Container) -> Self {
            let root = Rc::new(RefCell::new(root));
            let root_ref = Rc::downgrade(&root);
            Self {
                nodes: vec![root],
                root: root_ref.clone(),
                parents: vec![],
                children: vec![(root_ref, vec![])],
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FolderResource<T> {
    Present(T),
    Absent,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum FileResource {
    Present,
    Absent,
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

    pub fn is_present(&self) -> bool {
        match self {
            Self::Absent => false,
            Self::Present(_) => true,
        }
    }
}

pub type DataResource<T> = Result<T, IoSerde>;

pub(crate) mod action {
    use super::{
        analysis, container, graph, project::State as Project, DataResource, FolderResource,
    };
    use std::path::PathBuf;
    use syre_core::project::Project as CoreProject;
    use syre_local::types::ProjectSettings;

    #[derive(Debug, derive_more::From)]
    pub enum Action {
        /// Sets the project's path.
        SetPath(PathBuf),

        /// Sets the project's base folder to be `Absent`.
        RemoveFolder,

        /// Sets the project's base folder to be `Present` with the given state.
        CreateFolder(Project),

        /// Sets all config resources to be `Absent`.
        RemoveConfig,

        SetProperties(DataResource<CoreProject>),
        SetSettings(DataResource<ProjectSettings>),
        SetAnalyses(DataResource<Vec<analysis::State>>),

        /// Sets all analyses' file system resource to be absent.
        /// Used if the project's analysis directory is removed.
        SetAnalysesAbsent,

        #[from]
        Graph(Graph),
    }

    #[derive(Debug)]
    pub enum Graph {
        /// Sets the state of the graph.
        Set(FolderResource<graph::State>),

        /// Create the graph.
        Create(container::State),
    }
}
