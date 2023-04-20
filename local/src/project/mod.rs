//! Functionality and resources related to Thot Projects.
//!
//! This includes:
//! + Projects
//! + Containers
//! + Assets
//! + Script Associations
pub mod types;

#[cfg(feature = "fs")]
pub mod asset;

#[cfg(feature = "fs")]
pub mod container;

#[cfg(feature = "fs")]
pub mod project;

#[cfg(feature = "fs")]
pub mod resources;

#[cfg(feature = "fs")]
pub mod script;

/// Current project format standard.
pub static PROJECT_FORMAT: &str = "0.10.0";
