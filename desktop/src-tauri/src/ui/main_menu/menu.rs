//! Main menu UI and functionality.
use tauri::{CustomMenuItem, Menu, Submenu};

/// Build the main menu.
pub fn main_menu() -> Menu {
    // thot submenu
    let act_settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let sm_thot = Submenu::new("Thot", Menu::new().add_item(act_settings));

    // projects submenu
    let act_new_project = CustomMenuItem::new("new_project".to_string(), "New");
    let sm_project = Submenu::new("Project", Menu::new().add_item(act_new_project));

    // main menu
    Menu::new().add_submenu(sm_thot).add_submenu(sm_project)
}

#[cfg(test)]
#[path = "./menu_test.rs"]
mod menu_test;
