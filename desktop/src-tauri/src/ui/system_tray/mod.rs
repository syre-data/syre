//! System tray UI and functionality.
pub mod events;
pub mod menu;

// Re-exports
pub use events::handle_system_tray_event;
pub use menu::system_tray;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
