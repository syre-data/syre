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

mod project {
    use super::{Action, DataResource, Error, FolderResource};
    use syre_core::project::Project as CoreProject;
    use syre_local::{
        types::{AnalysisKind, ProjectSettings},
        TryReducible,
    };

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
