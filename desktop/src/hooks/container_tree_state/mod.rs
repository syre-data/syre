//! `ContainerTreeState` hooks.
pub mod container;

// Re-exports
pub use container::use_container;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
