//! Functionality and resources related to Thot Projects.
//!
//! This includes:
//! + Projects
//! + Containers
//! + Assets
//! + Script Associations
pub mod asset;
pub mod container;
pub mod project;
pub mod resources;
pub mod script;

/// Current project format version.
pub static PROJECT_FORMAT_VERSION: &str = "0.10.0";
