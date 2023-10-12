//! Startup functionality.
use crate::db::UpdateActor;
use std::sync::mpsc;
use std::thread;
use tauri::{App, Manager};

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // database updates
    let update_actor = UpdateActor::new(app.get_window("main").unwrap());
    thread::Builder::new()
        .name("update-actor".into())
        .spawn(move || update_actor.run())
        .unwrap();

    Ok(())
}

/// Launches the splashscreen.
fn splashscreen(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // get windows
    let w_splashscreen = app
        .get_window("splashscreen")
        .expect("could not get splashscreen");

    let w_main = app.get_window("main").expect("could not get main window");

    // run init in new task
    tauri::async_runtime::spawn(async move {
        // NOTE If sleep time is less than 150ms SIGBUS error occurs.
        std::thread::sleep(std::time::Duration::from_millis(250));
        // TODO: Load user settings.
        w_splashscreen
            .close()
            .expect("could not close splashscreen");

        w_main.show().expect("could not show main window");
    });

    Ok(())
}
