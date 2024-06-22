//! App state.
use super::{config::State as App, project::State as Project};
use crate::state::ManifestState;
pub use action::Action;
pub use error::Error;
use std::path::{Path, PathBuf};
use syre_core::system::User;
use syre_local::{Reducible, TryReducible};

/// Application state.
#[derive(Debug)]
pub struct State {
    app: App,
    projects: Vec<Project>,
}

impl State {
    pub fn new(
        user_manifest: ManifestState<User>,
        project_manifest: ManifestState<PathBuf>,
    ) -> Self {
        Self {
            app: App::new(user_manifest, project_manifest),
            projects: vec![],
        }
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn projects(&self) -> &Vec<Project> {
        &self.projects
    }

    /// Finds a project with the matching path.
    pub fn find_project_by_path(&self, path: impl AsRef<Path>) -> Option<&Project> {
        self.projects
            .iter()
            .find(|project| project.path() == path.as_ref())
    }

    /// Finds a project for the resource.
    pub fn find_resource_project_by_path(&self, path: impl AsRef<Path>) -> Option<&Project> {
        let path = path.as_ref();
        self.projects
            .iter()
            .find(|project| path.starts_with(project.path()))
    }
}

impl TryReducible for State {
    type Action = Action;
    type Error = Error;
    fn try_reduce(&mut self, action: Self::Action) -> Result<(), Self::Error> {
        match action {
            Action::Config(action) => {
                self.app.reduce(action);
                Ok(())
            }
            Action::InsertProject(project) => {
                self.projects.push(project);
                Ok(())
            }
            Action::RemoveProject(path) => {
                self.projects.retain(|project| project.path() != &path);
                Ok(())
            }
            Action::Project { path, action } => {
                let Some(project) = self
                    .projects
                    .iter_mut()
                    .find(|project| project.path() == &path)
                else {
                    return Err(Error::DoesNotExist);
                };

                project.try_reduce(action)
            }
        }
    }
}

mod action {
    use super::super::{
        config::Action as ConfigAction,
        project::{Action as ProjectAction, State as Project},
    };
    use std::path::PathBuf;

    #[derive(Debug, derive_more::From)]
    pub enum Action {
        #[from]
        Config(ConfigAction),
        InsertProject(Project),
        RemoveProject(PathBuf),
        Project {
            /// Path to the project's base folder.
            path: PathBuf,
            action: ProjectAction,
        },
    }
}

mod error {
    #[derive(Debug)]
    pub enum Error {
        /// The given action is invalid from the current state.
        InvalidTransition,

        /// The resource does not exist.
        DoesNotExist,
    }
}
