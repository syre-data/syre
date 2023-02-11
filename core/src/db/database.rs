//! Database of a Thot project.
use super::Collection;
use crate::project::{Asset, Container};
use crate::types::ResourceId;

#[derive(Debug)]
pub struct Database {
    pub root: ResourceId,
    pub containers: Collection<Container>,
    pub assets: Collection<Asset>,
}

impl Database {}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
