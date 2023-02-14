#![feature(io_error_more)]
#![feature(path_file_prefix)]
//! # Thot Local
//! This package contains local functionality and types of the Thot software suite.
pub mod common;
pub mod constants;
pub mod error;
pub mod system;
pub mod types;

#[cfg(feature = "fs")]
pub mod project;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;

// Re-exports
pub use error::{Error, Result};
