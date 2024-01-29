#![feature(io_error_more)]
#![feature(mutex_unlock)]
#![feature(path_file_prefix)]
//! # Syre Local
//! This package contains local functionality and types of the Syre software suite.
pub mod common;
pub mod constants;
pub mod error;
pub mod identifier;
pub mod loader;
pub mod system;
pub mod types;

#[cfg(feature = "fs")]
pub mod project;

#[cfg(feature = "fs")]
pub mod graph;

#[cfg(feature = "fs")]
pub mod file_resource;

// Re-exports
pub use error::{Error, Result};
