//! Project related components.
pub mod project;
pub mod set_data_root;

// Re-exports
pub use project::Project;
pub use set_data_root::SetDataRoot;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
