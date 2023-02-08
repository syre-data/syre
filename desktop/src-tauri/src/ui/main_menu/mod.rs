//! Main menu UI and functionality.
pub mod events;
pub mod menu;

// Re-exports
pub use events::handle_menu_event;
pub use menu::main_menu;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
