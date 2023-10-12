//! Database update messages.
//!
//! Topic should be `project:` followed by the resource id of the affected project.
//! e.g. `project:123-4567-890
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thot_core::types::ResourceId;

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
}

impl From<Container> for Project {
    fn from(update: Container) -> Self {
        Self::Container(update)
    }
}

// *****************
// *** Container ***
// *****************

/// Container updates.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Container {
    Properties {
        container: ResourceId,
        properties: thot_core::project::ContainerProperties,
    },
}
