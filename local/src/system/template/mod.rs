/// Templates.
pub mod project;

// Re-exports
pub use project::Project;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
