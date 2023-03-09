//! `ContainerTreeState` hooks.
// @remove
// pub mod container;
pub mod container_tree;

// Re-exports
// pub use container::use_container;
pub use container_tree::use_container_tree;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
