//! Database update messages.
//!
//! Topic should be `project:` followed by the resource id of the affected project, if known,
//! otherwise `unknown`.
//! e.g. `project/123-4567-890`, `project/unknown`
use crate::server::state::project::analysis;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::{
    graph::ResourceTree,
    project::{
        Container as CoreContainer, ContainerProperties, Project as CoreProject,
        Script as CoreScript,
    },
    types::ResourceId,
};
use syre_local::{
    error::IoSerde,
    types::{AnalysisKind, ProjectSettings},
};
use uuid::Uuid;

/// Update types.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Update {
    id: Uuid,
    parent: Uuid,
    kind: UpdateKind,
}

impl Update {
    pub fn project(
        id: Option<ResourceId>,
        path: impl Into<PathBuf>,
        update: Project,
        parent: Uuid,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            parent,
            kind: UpdateKind::Project {
                project: id,
                path: path.into(),
                update,
            },
        }
    }

    pub fn project_with_id(
        id: ResourceId,
        path: impl Into<PathBuf>,
        update: Project,
        parent: Uuid,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            parent,
            kind: UpdateKind::Project {
                project: Some(id),
                path: path.into(),
                update,
            },
        }
    }

    pub fn project_no_id(path: impl Into<PathBuf>, update: Project, parent: Uuid) -> Self {
        Self {
            id: Uuid::now_v7(),
            parent,
            kind: UpdateKind::Project {
                project: None,
                path: path.into(),
                update,
            },
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
    Project {
        project: Option<ResourceId>,
        path: PathBuf,
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
    Updated,
    Repaired,
    Corrupted,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProjectManifest {
    Added(Vec<PathBuf>),
    Removed(Vec<PathBuf>),
    Repaired,
    Corrupted,
}

#[derive(Serialize, Deserialize, Clone, Debug, derive_more::From)]
pub enum Project {
    Removed,
    Moved(PathBuf),
    Properties(DataResource<CoreProject>),
    Settings(DataResource<ProjectSettings>),
    Analyses(DataResource<Vec<analysis::State>>),

    #[from]
    Graph(Graph),

    #[from]
    Container(Container),

    #[from]
    Asset(Asset),

    #[from]
    AnalysisFile(AnalysisFile),

    Flag {
        resource: ResourceId,
        message: String,
    },
}

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

/// Container updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Container {
    /// `Container`'s properties were modified.
    Properties {
        container: ResourceId,
        properties: ContainerProperties,
    },
}

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

/// Analysis updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnalysisFile {
    Created(PathBuf),
    Removed(PathBuf),

    /// An `Analysis`'s path changed.
    ///
    /// # Notes
    /// + The `Analysis` remains in the same project.
    Moved {
        script: ResourceId,
        path: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DataResource<T> {
    Created(Result<T, IoSerde>),
    Removed,
    Corrupted(IoSerde),
    Repaired(T),
    Modified(T),
}
