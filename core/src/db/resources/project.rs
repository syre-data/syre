//! Project
use super::container::Container;

/// Project.
pub struct Project {
    pub data_root: Container,
    pub universal_root: Option<Container>,
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
