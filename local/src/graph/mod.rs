//! Local graphs.
pub mod tree;

// Re-exports
pub use tree::{ContainerTreeDuplicator, ContainerTreeLoader};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
