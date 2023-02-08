//! Handle system tray events.
use super::menu::SystemTrayItem;
use tauri::{AppHandle, SystemTrayEvent};

/// Handle system tray events.
pub fn handle_system_tray_event(_app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { tray_id: _, id, .. } => {
            let Some(id) = SystemTrayItem::from_id(&id) else {
                return;
            };

            match id {
                SystemTrayItem::Quit => {
                    std::process::exit(0);
                }
                SystemTrayItem::Discord => {
                    let url = "https://discord.gg/NCZUWWQqnd";
                    open::that(url).expect("could not open Discord");
                }
            }
        }
        SystemTrayEvent::LeftClick {
            tray_id: _,
            position: _,
            size: _,
            ..
        } => {}
        SystemTrayEvent::DoubleClick {
            tray_id: _,
            position: _,
            size: _,
            ..
        } => {}
        SystemTrayEvent::RightClick {
            tray_id: _,
            position: _,
            size: _,
            ..
        } => {}
        _ => {}
    }
}

#[cfg(test)]
#[path = "./events_test.rs"]
mod events_test;
