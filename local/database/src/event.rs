//! Database update messages.
//!
//! Topic should be `project:` followed by the resource id of the affected project, if known,
//! otherwise `unknown`.
//! e.g. `project/123-4567-890`, `project/unknown`
use crate::state;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use syre_core::{
    graph::ResourceTree,
    project::{Container as CoreContainer, ContainerProperties, Project as CoreProject},
    types::ResourceId,
};
use syre_local::{
    error::IoSerde,
    project::resources::container::StoredContainerProperties,
    types::{ContainerSettings, ProjectSettings},
};
use uuid::Uuid;

/// Update types.
#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    id: Uuid,
    parent: Uuid,
    kind: UpdateKind,
}

impl Update {
    /// # Arguments
    /// 1. `id`: Project id.
    /// 2. `path`: Project base path.
    /// 3. `update`
    /// 4. `parent`: Event's parent event id.
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

    /// # Arguments
    /// 1. `id`: Project id.
    /// 2. `path`: Project base path.
    /// 3. `update`
    /// 4. `parent`: Event's parent event id.
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

    /// # Arguments
    /// 1. `path`: Project base path.
    /// 2. `update`
    /// 3. `parent`: Event's parent event id.
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

    /// # Arguments
    /// 1. `update`
    /// 2. `parent`: Event's parent event id.
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

#[derive(Serialize, Deserialize, Debug)]
pub enum UpdateKind {
    App(App),
    Project {
        /// Project id.
        project: Option<ResourceId>,

        /// Project base path.
        path: PathBuf,
        update: Project,
    },
}

#[derive(Serialize, Deserialize, Debug, derive_more::From)]
pub enum App {
    UserManifest(UserManifest),
    ProjectManifest(ProjectManifest),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserManifest {
    Added,
    Removed,
    Updated,
    Repaired,
    Corrupted,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ProjectManifest {
    Added(Vec<PathBuf>),
    Removed(Vec<PathBuf>),
    Repaired,
    Corrupted,
}

#[derive(Serialize, Deserialize, Debug, derive_more::From)]
pub enum Project {
    Removed,
    Moved(PathBuf),
    Properties(DataResource<CoreProject>),
    Settings(DataResource<ProjectSettings>),
    Analyses(DataResource<Vec<state::Analysis>>),

    #[from]
    Graph(Graph),

    #[from]
    Container {
        /// Absolute path from the data root.
        ///
        /// # Notes
        /// Root container's path is `/`.
        path: PathBuf,
        update: Container,
    },

    #[from]
    Asset(AssetFile),

    #[from]
    AnalysisFile(AnalysisFile),

    Flag {
        resource: ResourceId,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Graph {
    /// The root graph was created.
    Created(state::Graph),

    /// A subgraph was created.
    Inserted {
        parent: ResourceId,
        graph: state::Graph,
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
#[derive(Serialize, Deserialize, Debug)]
pub enum Container {
    /// `Container`'s properties were modified.
    Properties(DataResource<StoredContainerProperties>),
    Settings(DataResource<ContainerSettings>),
    Assets(DataResource<Vec<state::Asset>>),
}

/// Asset updates.
#[derive(Serialize, Deserialize, Debug)]
pub enum AssetFile {
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
#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum DataResource<T> {
    Created(Result<T, IoSerde>),
    Removed,
    Corrupted(IoSerde),
    Repaired(T),
    Modified(T),
}
