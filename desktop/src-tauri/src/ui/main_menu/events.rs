//! Handle main menu events.
use crate::error::Result;
use settings_manager::system_settings::Loader;
use tauri::{Window, WindowMenuEvent};
use thot_local::system::{common::config_dir_path, settings::RunnerSettings};

/// Direct menu events to the correct function.
pub fn handle_menu_event(event: WindowMenuEvent) {
    let eid = event.menu_item_id();
    match eid {
        "settings" => {}
        "developer_settings" => {
            open_developer_settings(event.window()).expect("could not open settings")
        }
        _ => println!("Unhandled event {eid}"),
    }
}

/// Emit an `open_settings` event.
pub fn open_developer_settings(window: &Window) -> Result {
    Loader::load_or_create::<RunnerSettings>()?;
    match config_dir_path() {
        Ok(path) => {
            open::that(path)?;
        }
        Err(_) => {
            //TODO[l]: Display error message
        }
    };
    window.emit("thot://open-settings", ())?;
    Ok(())
}
