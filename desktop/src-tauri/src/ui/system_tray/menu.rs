//! System tray functionality.
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayMenuItem};

/// System tray items.
pub enum SystemTrayItem {
    Quit,
    Discord,
}

impl SystemTrayItem {
    /// Returns a tuple of (id, display).
    pub fn info(&self) -> (&'static str, &'static str) {
        match self {
            SystemTrayItem::Quit => ("quit", "Quit"),
            SystemTrayItem::Discord => ("discord", "Open Discord"),
        }
    }

    pub fn id(&self) -> &'static str {
        Self::info(&self).0
    }

    pub fn display(&self) -> &'static str {
        Self::info(&self).1
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "quit" => Some(Self::Quit),
            "discord" => Some(Self::Discord),
            _ => None,
        }
    }
}

/// Create the apps sytem tray.
pub fn system_tray() -> SystemTray {
    // menu items
    let quit = CustomMenuItem::new(SystemTrayItem::Quit.id(), SystemTrayItem::Quit.display());
    let discord = CustomMenuItem::new(
        SystemTrayItem::Discord.id(),
        SystemTrayItem::Discord.display(),
    );

    let tray_menu = SystemTrayMenu::new()
        .add_item(discord)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    // init tray
    SystemTray::new().with_menu(tray_menu)
}

#[cfg(test)]
#[path = "./menu_test.rs"]
mod menu_test;
