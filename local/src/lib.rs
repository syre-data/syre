#![feature(io_error_more)]
#![feature(mutex_unlock)]
#![feature(path_file_prefix)]
//! # Thot Local
//! This package contains local functionality and types of the Thot software suite.
pub mod common;
pub mod constants;
pub mod error;
pub mod project;
pub mod system;
pub mod types;
pub mod identifier;

#[cfg(feature = "fs")]
pub mod graph;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;

// Re-exports
pub use error::{Error, Result};
