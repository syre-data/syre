//! Project state.
use super::Error;
pub use action::Action;
use std::path::PathBuf;
use syre_local::{error::IoSerde, TryReducible};

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
            Action::CreateFolder(project) => todo!(),
            Action::RemoveConfig => {
                let FolderResource::Present(project) = self.fs_resource.as_mut() else {
                    return Err(Error::InvalidTransition);
                };

                project.try_reduce(action)
            }
        }
    }
}

pub mod project {
    use super::{Action, DataResource, Error, FolderResource};
    use std::io::ErrorKind;
    use syre_core::project::Project as CoreProject;
    use syre_local::{
        error::IoSerde,
        types::{AnalysisKind, ProjectSettings},
        TryReducible,
    };

    #[derive(Debug)]
    pub struct Builder {
        properties: DataResource<CoreProject>,
        settings: DataResource<ProjectSettings>,
        analyses: DataResource<Vec<AnalysisKind>>,
        graph: FolderResource<()>,
    }

    impl Builder {
        pub fn set_properties(&mut self, properties: CoreProject) {
            self.properties = DataResource::Ok(properties);
        }

        pub fn set_properties_err(&mut self, properties: impl Into<IoSerde>) {
            self.properties = DataResource::Err(properties.into());
        }

        pub fn set_settings(&mut self, settings: ProjectSettings) {
            self.settings = DataResource::Ok(settings);
        }

        pub fn set_settings_err(&mut self, settings: impl Into<IoSerde>) {
            self.settings = DataResource::Err(settings.into());
        }

        pub fn set_analyses(&mut self, analyses: Vec<AnalysisKind>) {
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

    #[derive(Debug)]
    pub struct State {
        properties: DataResource<CoreProject>,
        settings: DataResource<ProjectSettings>,
        analyses: DataResource<Vec<AnalysisKind>>,
        graph: FolderResource<()>,
    }

    impl TryReducible for State {
        type Action = Action;
        type Error = Error;
        fn try_reduce(&mut self, action: Self::Action) -> std::result::Result<(), Self::Error> {
            match action {
                Action::SetPath(_) | Action::RemoveFolder | Action::CreateFolder(_) => {
                    unreachable!("handled elsewhere");
                }

                Action::RemoveConfig => todo!(),
            }
        }
    }
}

#[derive(Debug)]
pub enum FolderResource<T> {
    Present(T),
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
}

pub type DataResource<T> = Result<T, IoSerde>;

mod action {
    use super::project::State as Project;
    use std::path::PathBuf;

    #[derive(Debug)]
    pub enum Action {
        /// Sets the project's path.
        SetPath(PathBuf),

        /// Sets the project's base folder to be `Absent`.
        RemoveFolder,

        /// Sets the project's base folder to be `Present` with the given state.
        CreateFolder(Project),

        /// Sets all config resources to be `Absent`.
        RemoveConfig,
    }
}
