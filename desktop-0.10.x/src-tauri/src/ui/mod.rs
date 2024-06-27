//! App resources.
pub mod main_menu;
pub mod system_tray;

// Re-exports
pub use main_menu::{handle_menu_event, main_menu};
pub use system_tray::{handle_system_tray_event, system_tray};
