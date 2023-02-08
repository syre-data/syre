//! Project workspace page.
pub mod page;
pub mod workspace;

// Re-exports
pub use page::Workspace;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
