//! Database update messages.
//!
//! Topic should be `project:` followed by the resource id of the affected project.
//! e.g. `project:123-4567-890
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::graph::ResourceTree;
use syre_core::project::{
    Container as CoreContainer, ContainerProperties, Project as CoreProject, Script as CoreScript,
};
use syre_core::types::ResourceId;
use uuid::Uuid;

/// Update types.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Update {
    id: Uuid,
    parent: Uuid,
    kind: UpdateKind,
}

impl Update {
    pub fn project(project: ResourceId, update: Project, parent: Uuid) -> Self {
        Self {
            id: Uuid::now_v7(),
            parent,
            kind: UpdateKind::Project { project, update },
        }
    }

    pub fn app(update: impl Into<App>, parent: Uuid) -> Self {
        Self {
            id: Uuid::now_v7(),
            parent,
            kind: UpdateKind::App(update.into()),
        }
    }
}

impl Update {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn kind(&self) -> &UpdateKind {
        &self.kind
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UpdateKind {
    App(App),
    ProjectCreated,
    Project {
        project: ResourceId,
        update: Project,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, derive_more::From)]
pub enum App {
    UserManifest(UserManifest),
    ProjectManifest(ProjectManifest),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserManifest {
    Added,
    Removed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProjectManifest {
    Added(Vec<PathBuf>),
    Removed(Vec<PathBuf>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Project {
    Removed(Option<CoreProject>),
    Moved(PathBuf),

    Graph(Graph),
    Container(Container),
    Asset(Asset),
    Script(Script),
    Analysis(Analysis),
}

impl From<Graph> for Project {
    fn from(update: Graph) -> Self {
        Self::Graph(update)
    }
}

impl From<Container> for Project {
    fn from(update: Container) -> Self {
        Self::Container(update)
    }
}

impl From<Asset> for Project {
    fn from(update: Asset) -> Self {
        Self::Asset(update)
    }
}

impl From<Script> for Project {
    fn from(update: Script) -> Self {
        Self::Script(update)
    }
}

impl From<Analysis> for Project {
    fn from(update: Analysis) -> Self {
        Self::Analysis(update)
    }
}

// *************
// *** Graph ***
// *************

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Graph {
    /// A subgraph was created.
    Created {
        parent: ResourceId,
        graph: ResourceTree<CoreContainer>,
    },

    /// A subgraph was moved within the `Project`.
    ///
    /// # Fields
    /// `parent`: New parent of the subgraph.
    /// `root`: Root of the subgraph.
    /// `name`: Name of the root `Container`.
    Moved {
        root: ResourceId,
        parent: ResourceId,
        name: String,
    },

    /// Subgraph was removed.
    Removed(ResourceTree<CoreContainer>),
}

// *****************
// *** Container ***
// *****************

/// Container updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Container {
    /// `Container`'s properties were modified.
    Properties {
        container: ResourceId,
        properties: ContainerProperties,
    },
}

// *************
// *** Asset ***
// *************

/// Asset updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Asset {
    Created {
        container: ResourceId,
        asset: syre_core::project::Asset,
    },

    /// The `Asset`'s path property changed.
    PathChanged {
        asset: ResourceId,
        path: PathBuf,
    },

    /// An Asset moved `Container`s.
    Moved {
        asset: ResourceId,
        container: ResourceId,
        path: PathBuf,
    },

    Removed(ResourceId),
}

// **************
// *** Script ***
// **************

/// Script updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Script {
    Created(CoreScript),
    Removed(ResourceId),

    /// A `Script`'s relative path changed.
    ///
    /// # Notes
    /// + The `Script` remains in the same project.
    Moved {
        script: ResourceId,
        path: PathBuf,
    },
}

// ****************
// *** Analysis ***
// ****************

/// Analysis updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Analysis {
    Flag {
        resource: ResourceId,
        message: String,
    },
}
