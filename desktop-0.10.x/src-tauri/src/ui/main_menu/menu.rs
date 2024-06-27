//! Main menu UI and functionality.
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

/// Build the main menu.
pub fn main_menu() -> Menu {
    // submenu
    let dev_settings = CustomMenuItem::new("developer_settings".to_string(), "Developer settings");
    let sm_syre = Submenu::new("Syre", Menu::new().add_item(dev_settings));

    // projects submenu
    let act_new_project = CustomMenuItem::new("new_project".to_string(), "New");
    let sm_project = Submenu::new("Project", Menu::new().add_item(act_new_project));

    let edit_menu = Submenu::new(
        "Edit",
        Menu::new()
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::SelectAll),
    );

    // main menu
    Menu::new()
        .add_submenu(sm_syre)
        .add_submenu(sm_project)
        .add_submenu(edit_menu)
}
