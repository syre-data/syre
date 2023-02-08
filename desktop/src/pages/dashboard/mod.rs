//! User dashboard.
pub mod dashboard;
pub mod page;

// Re-exports
pub use page::Dashboard;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
