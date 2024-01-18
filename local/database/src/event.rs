//! Database update messages.
//!
//! Topic should be `project:` followed by the resource id of the affected project.
//! e.g. `project:123-4567-890
use serde::{Deserialize, Serialize};
use thot_core::graph::ResourceTree;
use thot_core::project::{
    Container as CoreContainer, ContainerProperties, Project as CoreProject, Script as CoreScript,
};
use thot_core::types::{ResourceId, ResourcePath};

// **************
// *** Update ***
// **************

/// Update types.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Update {
    Project {
        project: ResourceId,
        update: Project,
    },
}

// ***************
// *** Project ***
// ***************

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Project {
    Removed(Option<CoreProject>),

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
        asset: thot_core::project::Asset,
    },

    /// The `Asset`'s path property changed.
    PathChanged {
        asset: ResourceId,
        path: ResourcePath,
    },

    /// An Asset moved `Container`s.
    Moved {
        asset: ResourceId,
        container: ResourceId,
        path: ResourcePath,
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
        path: ResourcePath,
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
