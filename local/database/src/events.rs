//! Database update messages.
//!
//! Topic should be `project:` followed by the resource id of the affected project.
//! e.g. `project:123-4567-890
use serde::{Deserialize, Serialize};
use thot_core::graph::ResourceTree;
use thot_core::project::{Container as CoreContainer, ContainerProperties};
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
    Graph(Graph),
    Container(Container),
    Asset(Asset),
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
    Moved {
        parent: ResourceId,
        root: ResourceId,
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

/// Container updates.
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
    },

    Removed(ResourceId),
}
