//! Sidebar
pub mod commands;
pub mod project_list;
pub mod script_list;
pub mod sidebar;

// Re-exports
pub use sidebar::Sidebar;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
