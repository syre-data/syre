//! Project and project settings.
use crate::types::{Creator, ResourceId};
use chrono::prelude::*;
use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ***************
// *** Project ***
// ***************

/// Represents a Thot project.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub rid: ResourceId,
    pub creator: Creator,
    pub created: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub data_root: Option<PathBuf>,
    pub universal_root: Option<PathBuf>,
    pub analysis_root: Option<PathBuf>,
    pub meta_level: u16,
}

impl Project {
    /// Creates a new Project.
    pub fn new(name: &str) -> Project {
        Project {
            rid: ResourceId::new(),
            name: String::from(name),
            creator: Creator::User(None),
            created: Utc::now(),
            description: None,
            data_root: None,
            universal_root: None,
            analysis_root: None,
            meta_level: 0,
        }
    }
}

impl Default for Project {
    fn default() -> Project {
        Project {
            rid: ResourceId::new(),
            name: String::from(""),
            creator: Creator::User(None),
            created: Utc::now(),
            description: None,
            data_root: None,
            universal_root: None,
            analysis_root: None,
            meta_level: 0,
        }
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
