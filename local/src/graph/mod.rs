//! Local graphs.
pub mod tree;

// Re-exports
pub use tree::ResourceTree;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
