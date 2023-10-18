//! Database update messages.
//!
//! Topic should be `project:` followed by the resource id of the affected project.
//! e.g. `project:123-4567-890
use serde::{Deserialize, Serialize};
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
    Container(Container),
    Asset(Asset),
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

// *****************
// *** Container ***
// *****************

/// Container updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Container {
    /// `Container`'s properties were modified.
    Properties {
        container: ResourceId,
        properties: thot_core::project::ContainerProperties,
    },

    /// A child `Container` was created.
    ChildCreated {
        parent: ResourceId,
        container: thot_core::project::Container,
    },

    /// `Container`` was removed.
    Removed(ResourceId),
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

    PathChanged {
        asset: ResourceId,
        path: ResourcePath,
    },

    Removed(ResourceId),
}
