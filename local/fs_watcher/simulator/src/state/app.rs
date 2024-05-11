use super::{
    actions,
    graph::{self, Tree},
};
use has_id::HasId;
use std::{ops::Deref, path::PathBuf};
use syre_core::types::ResourceId;

#[derive(Clone, Default, Debug)]
pub struct State {
    pub user_manifest: Resource,
    pub project_manifest: Resource,
    pub projects: Vec<Project>,
}

impl State {
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

impl State {
    pub fn transition(&mut self, action: &actions::Action) -> Result<(), error::Transition> {
        use actions::Action;
        match action {
            Action::App(actions::AppResource::UserManifest(action)) => {
                self.handle_action_user_manifest(action)
            }

            Action::App(actions::AppResource::ProjectManifest(action)) => {
                self.handle_action_project_manifest(action)
            }

            Action::CreateProject { id, path } => {
                self.projects
                    .push(Project::with_id(path.clone(), id.clone()));

                Ok(())
            }

            Action::Project { project, action } => {
                self.handle_action_project_resource(&project, &action)
            }

            Action::Watch(_) => Ok(()),
            Action::Unwatch(_) => Ok(()),
        }
    }

    fn handle_action_user_manifest(
        &mut self,
        action: &actions::Manifest,
    ) -> Result<(), error::Transition> {
        match action {
            actions::Manifest::Create => match self.user_manifest {
                Resource::NotPresent => {
                    self.user_manifest = Resource::Valid(());
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

            actions::Manifest::Rename => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.user_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Move => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.user_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Copy => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    return Ok(());
                }
            },

            actions::Manifest::Corrupt => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => Err(error::Transition::AlreadyInState),
                Resource::Valid(_) => {
                    self.user_manifest = Resource::Invalid;
                    Ok(())
                }
            },

            actions::Manifest::Repair => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => {
                    self.user_manifest = Resource::Valid(());
                    Ok(())
                }
                Resource::Valid(_) => Err(error::Transition::AlreadyInState),
            },

            actions::Manifest::Modify(kind) => match self.user_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => Ok(()),
            },
        }
    }

    fn handle_action_project_manifest(
        &mut self,
        action: &actions::Manifest,
    ) -> Result<(), error::Transition> {
        match action {
            actions::Manifest::Create => match self.project_manifest {
                Resource::NotPresent => {
                    self.project_manifest = Resource::Valid(());
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

            actions::Manifest::Rename => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.project_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Move => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    self.project_manifest = Resource::NotPresent;
                    Ok(())
                }
            },

            actions::Manifest::Copy => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => {
                    return Ok(());
                }
            },

            actions::Manifest::Corrupt => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => Err(error::Transition::AlreadyInState),
                Resource::Valid(_) => {
                    self.project_manifest = Resource::Invalid;
                    Ok(())
                }
            },

            actions::Manifest::Repair => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                Resource::Invalid => {
                    self.project_manifest = Resource::Valid(());
                    Ok(())
                }
                Resource::Valid(_) => Err(error::Transition::AlreadyInState),
            },

            actions::Manifest::Modify(kind) => match self.project_manifest {
                Resource::NotPresent => Err(error::Transition::InvalidAction),
                _ => Ok(()),
            },
        }
    }

    fn handle_action_project_resource(
        &mut self,
        pid: &ResourceId,
        action: &actions::ProjectResource,
    ) -> Result<(), error::Transition> {
        use super::actions::{Project, ProjectResource, ResourceDir, StaticDir};

        let project = self.find_project_mut(pid);
        match action {
            ProjectResource::Project(action) => match action {
                Project::Project(action) => match action {
                    ResourceDir::Remove => {
                        self.remove_project(pid).unwrap();
                        Ok(())
                    }

                    ResourceDir::Rename { to } => {
                        project.unwrap().path = to.clone();
                        Ok(())
                    }

                    ResourceDir::Move { to } => {
                        project.unwrap().path = to.clone();
                        Ok(())
                    }

                    ResourceDir::Copy { to } => Ok(()),
                },

                Project::ConfigDir(StaticDir::Remove)
                | Project::ConfigDir(StaticDir::Rename)
                | Project::ConfigDir(StaticDir::Move) => {
                    project.unwrap().config = Reference::NotPresent;
                    Ok(())
                }

                _ => {
                    let project = project.unwrap();
                    Self::handle_action_project(project, action)
                }
            },

            ProjectResource::CreateContainer { parent, name } => {
                todo!();
            }

            ProjectResource::Container { container, action } => {
                let project = project.unwrap();
                match &project.data {
                    Reference::NotPresent => Err(error::Transition::InvalidAction),
                    Reference::Present(graph) => {
                        let container = graph
                            .nodes()
                            .iter()
                            .find(|node| node.borrow().rid() == container);

                        Self::handle_action_container(container, action)
                    }
                }
            }

            ProjectResource::CreateAssetFile { container, name } => todo!(),

            ProjectResource::AssetFile {
                container,
                asset,
                action,
            } => {
                let project = project.unwrap();
                match &project.data {
                    Reference::NotPresent => Err(error::Transition::InvalidAction),
                    Reference::Present(graph) => {
                        let container = graph
                            .nodes()
                            .iter()
                            .find(|node| node.borrow().rid() == container)
                            .unwrap();

                        Self::handle_action_asset_file(container, action)
                    }
                }
            }
        }
    }

    fn handle_action_project(
        project: &mut Project,
        action: &actions::Project,
    ) -> Result<(), error::Transition> {
        use actions::{Dir, Project, StaticDir};

        match action {
            Project::Project(_) => unreachable!("handled elsewhere"),
            Project::ConfigDir(action) => match project.config {
                Reference::NotPresent => match action {
                    StaticDir::Create => {
                        project.config = Reference::Present(ProjectConfig::default());
                        Ok(())
                    }

                    _ => Err(error::Transition::InvalidAction),
                },

                Reference::Present(_) => match action {
                    StaticDir::Create => Err(error::Transition::AlreadyInState),
                    StaticDir::Remove | StaticDir::Rename | StaticDir::Move => {
                        unreachable!("handled elsewhere")
                    }

                    StaticDir::Copy => Ok(()),
                },
            },

            Project::AnalysisDir(action) => match action {
                Dir::Create { path } => {
                    project.analyses = Some(Reference::Present(path.clone()));
                    Ok(())
                }

                Dir::Remove => {
                    assert!(project.analyses.is_some());
                    project.analyses = None;
                    Ok(())
                }

                Dir::Rename { to } => {
                    assert!(project.analyses.is_some());
                    project.analyses = Some(Reference::Present(to.clone()));
                    Ok(())
                }

                Dir::Move { to } => {
                    assert!(project.analyses.is_some());
                    project.analyses = Some(Reference::Present(to.clone()));
                    Ok(())
                }

                Dir::Copy { .. } => {
                    assert!(project.analyses.is_some());
                    Ok(())
                }
            },

            Project::DataDir(action) => match action {
                Dir::Create { path } => {
                    project.data = Reference::Present(Data::new(path));
                    Ok(())
                }

                Dir::Remove => {
                    project.data = Reference::NotPresent;
                    Ok(())
                }

                Dir::Rename { to } => Ok(()),

                Dir::Move { to } => Ok(()),

                Dir::Copy { .. } => Ok(()),
            },

            Project::Properties(action) => match action {
                _ => todo!(),
            },

            Project::Settings(action) => match action {
                _ => todo!(),
            },

            Project::Analyses(action) => match action {
                _ => todo!(),
            },
        }
    }

    fn handle_action_container(
        container: Option<&graph::Node<Container>>,
        action: &actions::Container,
    ) -> Result<(), error::Transition> {
        todo!()
    }

    fn handle_action_asset_file(
        container: &graph::Node<Container>,
        action: &actions::AssetFile,
    ) -> Result<(), error::Transition> {
        todo!()
    }
}

/// State of a configuration resource.
#[derive(Clone, Debug)]
pub enum Resource<T = ()> {
    /// The resource is valid and presesnt.
    Valid(T),

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

    /// Path to the project's base directory.
    pub path: PathBuf,

    pub config: Reference<ProjectConfig>,

    /// Analyses directory.
    /// `Option` variant matches that set by the project.
    pub analyses: Option<Reference<PathBuf>>,
    pub data: Reference<Data>,
}

impl Project {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            rid: ResourceId::new(),
            path: path.into(),
            config: Reference::default(),
            analyses: None,
            data: Reference::<Data>::default(),
        }
    }

    pub fn with_id(path: impl Into<PathBuf>, rid: ResourceId) -> Self {
        Self {
            rid,
            path: path.into(),
            config: Reference::default(),
            analyses: None,
            data: Reference::<Data>::default(),
        }
    }

    pub fn rid(&self) -> &ResourceId {
        &self.rid
    }
}

#[derive(Clone, Default, Debug)]
pub struct ProjectConfig {
    pub properties: Resource,
    pub settings: Resource,
    pub analyses: Resource,
}

#[derive(Clone, Default, Debug)]
pub struct ProjectProperties {
    state: Resource,
}

#[derive(Debug)]
pub struct Data {
    pub graph: Tree<Container>,
}

impl Data {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            graph: Tree::new(Container::new(path)),
        }
    }

    pub fn root_path(&self) -> PathBuf {
        self.root().borrow().path.clone()
    }
}

impl Clone for Data {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.duplicate(),
        }
    }
}

impl Deref for Data {
    type Target = Tree<Container>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

#[derive(Clone, Debug, HasId)]
pub struct Container {
    #[id]
    rid: ResourceId,
    pub path: PathBuf,
    pub config: Reference<ContainerConfig>,
}

impl Container {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            rid: ResourceId::new(),
            path: path.into(),
            config: Reference::NotPresent,
        }
    }

    pub fn with_id(path: impl Into<PathBuf>, rid: ResourceId) -> Self {
        Self {
            rid,
            path: path.into(),
            config: Reference::NotPresent,
        }
    }

    pub fn rid(&self) -> &ResourceId {
        &self.rid
    }

    pub fn find_asset(&self, id: &ResourceId) -> Option<&Asset> {
        let Reference::Present(config) = &self.config else {
            return None;
        };

        let Resource::Valid(assets) = &config.assets else {
            return None;
        };

        assets.iter().find(|asset| asset.rid() == id)
    }
}

#[derive(Clone, Debug)]
pub struct ContainerConfig {
    pub properties: Resource,
    pub settings: Resource,
    pub assets: Resource<Vec<Asset>>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            properties: Resource::NotPresent,
            settings: Resource::NotPresent,
            assets: Resource::NotPresent,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Asset {
    rid: ResourceId,

    /// The
    pub path: PathBuf,

    /// Whether the referenced file is present.
    pub file: Reference,
}

impl Asset {
    pub fn new(rid: ResourceId, path: impl Into<PathBuf>) -> Self {
        Self {
            rid,
            path: path.into(),
            file: Reference::NotPresent,
        }
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
