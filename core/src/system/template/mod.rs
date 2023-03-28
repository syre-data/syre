//! Templates.
pub mod project;
pub mod tree;

// Re-exports
pub use project::Project;
pub use tree::ResourceTree;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
