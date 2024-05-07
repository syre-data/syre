use super::{
    actions,
    graph::{self, Tree},
};
use std::{ops::Deref, path::PathBuf};
use syre_core::types::ResourceId;

#[derive(Clone, Default, Debug)]
pub struct App {
    pub user_manifest: Resource,
    pub project_manifest: Resource,
    pub watched: Vec<PathBuf>,
    pub projects: Vec<Project>,
}

impl App {
    pub fn find_project(&self, rid: &ResourceId) -> Option<&Project> {
        self.projects.iter().find_map(|project| {
            if &project.rid == rid {
                Some(project)
            } else {
                None
            }
        })
    }

    pub fn find_project_mut(&mut self, rid: &ResourceId) -> Option<&mut Project> {
        self.projects.iter_mut().find_map(|project| {
            if &project.rid == rid {
                Some(project)
            } else {
                None
            }
        })
    }

    pub fn remove_project(&mut self, rid: &ResourceId) -> Option<Project> {
        let index = self
            .projects
            .iter()
            .position(|project| project.rid() == rid)?;

        Some(self.projects.swap_remove(index))
    }
}

impl App {
    pub fn transition(&mut self, action: &actions::Action) -> Result<(), error::Transition> {
        match action {
            actions::Action::App(actions::AppResource::UserManifest(action)) => {
                self.handle_user_manifest_action(action)
            }

            actions::Action::App(actions::AppResource::ProjectManifest(action)) => {
                self.handle_project_manifest_action(action)
            }

            actions::Action::Project { project, action } => {
                self.handle_project_resource_action(&project, &action)
            }

            actions::Action::Watch(path) => {
                if self.watched.iter().any(|p| p == path) {
                    return Err(error::Transition::AlreadyInState);
                }

                self.watched.push(path.clone());
                Ok(())
            }

            actions::Action::Unwatch(path) => {
                let Some(index) = self.watched.iter().position(|p| p == path) else {
                    return Err(error::Transition::AlreadyInState);
                };

                self.watched.swap_remove(index);
                Ok(())
            }
        }
    }

    fn handle_user_manifest_action(
        &mut self,
        action: &actions::Manifest,
    ) -> Result<(), error::Transition> {
        match action {
            actions::Manifest::Create => match self.user_manifest {
                Resource::NotPresent => {
                    self.user_manifest = Resource::Valid;
                    Ok(())
                }
                _ => Err(error::Transition::InvalidAction),
            },

            actions::Manifest::Remove => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.user_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Rename(_) => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.user_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Move(_) => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.user_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Copy(_) => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    return Ok(());
                }
            },

            actions::Manifest::Corrupt => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => Err(error::Transition::AlreadyInState),
                Resource::Valid => {
                    self.user_manifest = Resource::Invalid;
                    Ok(())
                }
            },

            actions::Manifest::Repair => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => {
                    self.user_manifest = Resource::Valid;
                    Ok(())
                }
                Resource::Valid => Err(error::Transition::AlreadyInState),
            },

            actions::Manifest::Modify(_) => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => Ok(()),
            },
        }
    }

    fn handle_project_manifest_action(
        &mut self,
        action: &actions::Manifest,
    ) -> Result<(), error::Transition> {
        match action {
            actions::Manifest::Create => match self.project_manifest {
                Resource::NotPresent => {
                    self.project_manifest = Resource::Valid;
                    Ok(())
                }
                _ => Err(error::Transition::InvalidAction),
            },

            actions::Manifest::Remove => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.project_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Rename(_) => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.project_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Move(_) => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.project_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Copy(_) => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    return Ok(());
                }
            },

            actions::Manifest::Corrupt => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => Err(error::Transition::AlreadyInState),
                Resource::Valid => {
                    self.project_manifest = Resource::Invalid;
                    Ok(())
                }
            },

            actions::Manifest::Repair => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => {
                    self.project_manifest = Resource::Valid;
                    Ok(())
                }
                Resource::Valid => Err(error::Transition::AlreadyInState),
            },

            actions::Manifest::Modify(_) => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => Ok(()),
            },
        }
    }

    fn handle_project_resource_action(
        &mut self,
        pid: &ResourceId,
        action: &actions::ProjectResource,
    ) -> Result<(), error::Transition> {
        let project = self.find_project_mut(pid);
        match action {
            actions::ProjectResource::Project(action) => {
                // when creating a new project, it will not yet be in the state,
                // otherwise it should be.
                match action {
                    actions::Project::Project(action) => match action {
                        actions::Dir::Create(path) => {
                            self.projects
                                .push((Project::with_id(pid.clone()), path.clone()));

                            Ok(())
                        }

                        actions::Dir::Remove => {
                            self.remove_project(pid).unwrap();
                            Ok(())
                        }

                        actions::Dir::Rename(path) => {
                            let Some(watched) = self.watched.iter_mut().find(|p| *p == path) else {
                                return Err(error::Transition::InvalidAction);
                            };

                            *watched = path.to_path_buf();
                            Ok(())
                        }

                        actions::Dir::Move(path) => {
                            let Some(watched) = self.watched.iter_mut().find(|p| *p == path) else {
                                return Err(error::Transition::InvalidAction);
                            };

                            *watched = path.to_path_buf();
                            Ok(())
                        }

                        actions::Dir::Copy(_) => Ok(()),
                    },

                    actions::Project::ConfigDir(actions::StaticDir::Remove)
                    | actions::Project::ConfigDir(actions::StaticDir::Rename(_))
                    | actions::Project::ConfigDir(actions::StaticDir::Move(_)) => {
                        self.remove_project(pid).unwrap();
                        Ok(())
                    }

                    _ => {
                        let project = project.unwrap();
                        Self::handle_project_action(project, action)
                    }
                }
            }

            actions::ProjectResource::Container { container, action } => {
                let project = project.unwrap();
                match &project.graph {
                    Reference::NotPresent => Err(error::Transition::InvalidAction),
                    Reference::Present(graph) => {
                        let container = graph
                            .nodes()
                            .iter()
                            .find(|node| node.borrow().rid() == container);

                        Self::handle_container_action(container, action)
                    }
                }
            }

            actions::ProjectResource::AssetFile {
                container,
                asset,
                action,
            } => {
                let project = project.unwrap();
                match &project.graph {
                    Reference::NotPresent => Err(error::Transition::InvalidAction),
                    Reference::Present(graph) => {
                        let container = graph
                            .nodes()
                            .iter()
                            .find(|node| node.borrow().rid() == container)
                            .unwrap();

                        Self::handle_asset_file_action(container, action)
                    }
                }
            }
        }
    }

    fn handle_project_action(
        project: &mut Project,
        action: &actions::Project,
    ) -> Result<(), error::Transition> {
        match action {
            actions::Project::Project(_) => unreachable!("handled elsewhere"),
            actions::Project::ConfigDir(action) => match project.config {
                Reference::NotPresent => match action {
                    actions::StaticDir::Create => {
                        project.config = Reference::Present(ProjectConfig::default());
                        Ok(())
                    }

                    _ => Err(error::Transition::InvalidAction),
                },

                Reference::Present(_) => match action {
                    actions::StaticDir::Create => Err(error::Transition::AlreadyInState),
                    actions::StaticDir::Remove
                    | actions::StaticDir::Rename(_)
                    | actions::StaticDir::Move(_) => unreachable!("handled elsewhere"),

                    actions::StaticDir::Copy(_) => Ok(()),
                },
            },

            actions::Project::AnalysisDir(action) => match action {
                _ => todo!(),
            },

            actions::Project::DataDir(action) => match action {
                _ => todo!(),
            },

            actions::Project::Properties(action) => match action {
                _ => todo!(),
            },

            actions::Project::Settings(action) => match action {
                _ => todo!(),
            },

            actions::Project::Analyses(action) => match action {
                _ => todo!(),
            },
        }
    }

    fn handle_container_action(
        container: Option<&graph::Node<Container>>,
        action: &actions::Container,
    ) -> Result<(), error::Transition> {
        todo!()
    }

    fn handle_asset_file_action(
        container: &graph::Node<Container>,
        action: &actions::AssetFile,
    ) -> Result<(), error::Transition> {
        todo!()
    }
}

/// State of a configuration resource.
#[derive(Clone, Debug)]
pub enum Resource {
    /// The resource is valid and presesnt.
    Valid,

    /// The resource is present, but invalid.
    Invalid,

    /// The resource is not present.
    NotPresent,
}

impl Default for Resource {
    fn default() -> Self {
        Self::NotPresent
    }
}

/// The state of a referenced resource.
#[derive(Clone, Debug)]
pub enum Reference<R = ()> {
    Present(R),
    NotPresent,
}

impl<R> Default for Reference<R> {
    fn default() -> Self {
        Self::NotPresent
    }
}

#[derive(Clone, Debug)]
pub struct Project {
    rid: ResourceId,
    pub path: PathBuf,
    pub config: Reference<ProjectConfig>,

    /// Analyses directory.
    /// `Option` variant matches that set by the project.
    pub analyses: Option<Reference>,
    pub graph: Reference<Graph>,
}

impl Project {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(rid: ResourceId) -> Self {
        Self {
            rid,
            ..Default::default()
        }
    }

    pub fn rid(&self) -> &ResourceId {
        &self.rid
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            rid: ResourceId::new(),
            config: Reference::default(),
            analyses: None,
            graph: Reference::<Graph>::default(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct ProjectConfig {
    pub properties: Resource,
    pub settings: Resource,
    pub analyses: Resource,
}

#[derive(Debug)]
pub struct Graph {
    pub root_path: PathBuf,
    pub inner: Tree<Container>,
}

impl Clone for Graph {
    fn clone(&self) -> Self {
        Self {
            root_path: self.root_path.clone(),
            inner: self.inner.duplicate(),
        }
    }
}

impl Deref for Graph {
    type Target = Tree<Container>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug)]
pub struct Container {
    rid: ResourceId,
    pub path: PathBuf,
    pub config: Reference<ContainerConfig>,
    pub assets: Vec<Asset>,
}

impl Container {
    pub fn new(rid: ResourceId, config: Reference<ContainerConfig>) -> Self {
        Self {
            rid,
            config,
            assets: vec![],
        }
    }

    pub fn rid(&self) -> &ResourceId {
        &self.rid
    }
}

#[derive(Clone, Default, Debug)]
pub struct ContainerConfig {
    pub properties: Resource,
    pub settings: Resource,
    pub assets: Resource,
}

#[derive(Clone, Debug)]
pub struct Asset {
    rid: ResourceId,

    /// Whether the referenced file is present.
    pub file: Reference,
}

impl Asset {
    pub fn new(rid: ResourceId, file: Reference) -> Self {
        Self { rid, file }
    }

    pub fn rid(&self) -> &ResourceId {
        &self.rid
    }
}

pub mod error {
    #[derive(Debug)]
    pub enum Transition {
        /// The action is not valid given the current state.
        InvalidAction,

        /// Calling the action would not tranform the state.
        AlreadyInState,
    }
}
