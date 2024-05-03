use crate::resources::app;
use syre_core::types::ResourceId;

#[derive(Default, Debug)]
pub struct App {
    pub user_manifest: Resource,
    pub project_manifest: Resource,
    pub projects: Vec<Project>,
}

impl App {
    pub fn new(user_manifest: Resource, project_manifest: Resource) -> Self {
        Self {
            user_manifest,
            project_manifest,
            projects: vec![],
        }
    }

    pub fn valid(&self) -> bool {
        true
    }

    pub fn valid_if(self, action: app::Action) -> bool {
        true
    }
}

/// State of a configuration resource.
#[derive(Debug)]
pub enum Resource {
    /// The resource is valid and presesnt.
    Vaild,

    /// The resource is present, but invalid.
    Invaild,

    /// The resource is not present.
    NotPresent,
}

impl Default for Resource {
    fn default() -> Self {
        Self::NotPresent
    }
}

/// The state of a referenced resource.
#[derive(Debug)]
pub enum Reference<R = ()> {
    Present(R),
    NotPresent,
}

impl Default for Reference {
    fn default() -> Self {
        Self::NotPresent
    }
}

#[derive(Debug)]
pub struct Project {
    rid: ResourceId,
    pub config: Reference<ProjectConfig>,

    /// Whether the analysis directory is present.
    pub analysis: Reference,
    pub graph: Reference<Graph>,
}

impl Project {
    pub fn new(
        rid: ResourceId,
        config: Reference<ProjectConfig>,
        analysis: Reference,
        graph: Reference<Graph>,
    ) -> Self {
        Self {
            rid,
            config,
            analysis,
            graph,
        }
    }
}

#[derive(Default, Debug)]
pub struct ProjectConfig {
    pub properties: Resource,
    pub settings: Resource,
    pub analysis: Resource,
}

#[derive(Debug)]
pub struct Graph {
    pub containers: Vec<Container>,
}

#[derive(Debug)]
pub struct Container {
    rid: ResourceId,
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

#[derive(Default, Debug)]
pub struct ContainerConfig {
    pub properties: Resource,
    pub settings: Resource,
    pub assets: Resource,
}

#[derive(Debug)]
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

pub trait Valid {
    fn is_valid(&self) -> bool;
}
