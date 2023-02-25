#![feature(io_error_more)]
#![feature(assert_matches)]
// @todo: `is_sorted` only needed for testing.
#![feature(is_sorted)]

//! # Thot Core
//!
//! This package represents core functionality of the Thot software suite.
pub mod api;
pub mod common;
pub mod constants;
pub mod error;
pub mod identifier;
pub mod lib_impl;
pub mod types;
pub mod graph;

#[cfg(feature = "project")]
pub mod project;

#[cfg(feature = "db")]
pub mod db;

#[cfg(feature = "runner")]
pub mod runner;

#[cfg(feature = "system")]
pub mod system;

#[cfg(test)]
pub mod dev_utils;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;

// Re-exports
pub use error::{Error, Result};
