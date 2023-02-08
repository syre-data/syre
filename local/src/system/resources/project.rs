//! Project map.
use has_id::{HasId, HasIdSerde};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::path::PathBuf;
use thot_core::types::ResourceId;

#[derive(Serialize, Deserialize, HasId, HasIdSerde, Debug, PartialEq, Clone)]
pub struct Project {
    #[id]
    pub rid: ResourceId,
    pub path: PathBuf,
}

impl Project {
    pub fn new(rid: ResourceId, path: PathBuf) -> Project {
        Project { rid, path }
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
