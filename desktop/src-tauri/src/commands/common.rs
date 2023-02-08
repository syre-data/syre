//! Common functionality.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tauri::api::dialog::FileDialogBuilder;

/// Get a user selected directory path.
/// Returns `None` if the user cancels the action.
///
/// # Arguments
/// + `title`: Title of the dialog box, or `None` to use default.
/// + `dir`: Starting directory of the dialog, or `None` to use default.
#[tauri::command]
pub fn get_directory(title: Option<String>, dir: Option<PathBuf>) -> Option<PathBuf> {
    let path = Arc::new(Mutex::new(None));
    let action = Arc::new(Mutex::new(false));
    {
        let path = path.clone();
        let action = action.clone();

        let mut dlg = FileDialogBuilder::new();
        if let Some(title) = title {
            dlg = dlg.set_title(&title);
        }
        if let Some(dir) = dir {
            dlg = dlg.set_directory(&dir);
        }

        dlg.pick_folder(move |p| {
            let mut path = path.lock().unwrap();
            let mut action = action.lock().unwrap();

            *path = p;
            *action = true;
        });
    }

    let user_path;
    loop {
        // wait for user
        if *action.lock().unwrap() {
            user_path = path.lock().unwrap();
            break;
        }

        dbg!("waiting for user");
        sleep(Duration::from_millis(1000));
    }

    (*user_path).clone()
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
