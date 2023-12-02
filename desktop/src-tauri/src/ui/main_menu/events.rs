//! Handle main menu events.
use crate::error::Result;
use std::path::Path;
use tauri::{Window, WindowMenuEvent};
use thot_local::file_resource::SystemResource;
use thot_local::system::settings::RunnerSettings;

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
pub fn open_developer_settings(_window: &Window) -> Result {
    let path = RunnerSettings::path();
    if !Path::exists(&path) {
        let settings = RunnerSettings::default();
        settings.save()?;
    }

    Ok(open::that(path)?)
}
