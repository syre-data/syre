//! Module for loading a Thot project.
// pub mod collection;
// pub mod database;
// pub mod error;
pub mod resource;
pub mod search_filter;

// Re-exports
// pub use collection::Collection;
// pub use database::Database;
// pub use error::Error;
pub use resource::{Resource, StandardResource};
pub use search_filter::{SearchFilter, StandardSearchFilter};

#[cfg(test)]
mod dev_utils;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
