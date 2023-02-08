//! A template.
use crate::types::{ResourceId, ResourcePath};
use chrono::prelude::*;
use has_id::HasId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

// @todo: Separate template for Project, Container, Asset, and Script?
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, HasIdSerde))]
#[derive(HasId, Debug)]
pub struct Template {
    #[id]
    pub rid: ResourceId,
    pub parent: Option<ResourceId>,

    /// structure should be an abstract representation of the template
    pub template: ResourcePath,

    pub name: String,
    pub description: String,
    pub created: DateTime<Utc>,
    pub creator: Option<ResourceId>,
}
