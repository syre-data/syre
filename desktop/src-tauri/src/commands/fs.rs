use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle, title: String) -> Option<PathBuf> {
    app.dialog().file().set_title(title).blocking_pick_folder()
}

#[tauri::command]
pub async fn pick_folder_with_location(
    app: tauri::AppHandle,
    title: String,
    dir: PathBuf,
) -> Option<PathBuf> {
    app.dialog()
        .file()
        .set_title(title)
        .set_directory(dir)
        .blocking_pick_folder()
}
