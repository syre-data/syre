//! Handle main menu events.
use crate::error::Result;
use tauri::{Window, WindowMenuEvent};

/// Direct menu events to the correct function.
pub fn handle_menu_event(event: WindowMenuEvent) {
    let eid = event.menu_item_id();
    match eid {
        "settings" => open_settings(event.window()).expect("could not open settings"),
        _ => println!("Unhandled event {eid}"),
    }
}

/// Emit an `open_settings` event.
pub fn open_settings(window: &Window) -> Result {
    window.emit("thot://open-settings", ())?;
    Ok(())
}

#[cfg(test)]
#[path = "./events_test.rs"]
mod events_test;
