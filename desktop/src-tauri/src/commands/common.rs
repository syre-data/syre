//! Common functionality.
use crate::error::Result;
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

        sleep(Duration::from_millis(100));
    }

    (*user_path).clone()
}

#[tauri::command]
#[tracing::instrument(level = "debug")]
pub fn open_file(path: PathBuf) -> Result {
    let path = path.canonicalize()?;
    open::that(path).map_err(|err| err.into())
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
