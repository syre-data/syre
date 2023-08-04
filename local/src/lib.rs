#![feature(io_error_more)]
#![feature(mutex_unlock)]
#![feature(path_file_prefix)]
//! # Thot Local
//! This package contains local functionality and types of the Thot software suite.
pub mod common;
pub mod constants;
pub mod error;
pub mod identifier;
pub mod project;
pub mod system;
pub mod types;

#[cfg(feature = "fs")]
pub mod graph;

// Re-exports
pub use error::{Error, Result};
