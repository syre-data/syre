//! A [`Project`](crate::project::Project) template.
use crate::types::{ResourceId, ResourcePath};
use chrono::prelude::*;
use has_id::HasId;
use std::collections::HashSet;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, HasIdSerde))]
#[derive(HasId, Debug)]
pub struct Project {
    #[id]
    pub rid: ResourceId,

    /// Projects derived from the template.
    pub projects: HashSet<ResourceId>,

    /// Path to the template directory.
    /// The directory contains the project file and graph,
    /// and scripts.
    pub path: ResourcePath,

    pub name: String,
    pub description: String,
    pub created: DateTime<Utc>,
    pub creator: Option<ResourceId>,
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
