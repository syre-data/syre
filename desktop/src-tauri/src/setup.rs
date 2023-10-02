//! Startup functionality.
use crate::db::{UpdateActor, UpdateActorHandle};
use std::sync::mpsc;
use std::thread;
use tauri::{App, Manager};

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // database updates
    let (update_tx, update_rx) = mpsc::channel();
    let update_actor = UpdateActor::new(update_tx);
    let update_actor_handler = UpdateActorHandle::new(update_rx, app.get_window("main").unwrap());
    thread::spawn(move || update_actor.run());
    thread::spawn(move || update_actor_handler.run());

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
        // Important! If sleep time is less than 150ms SIGBUS error occurs.
        std::thread::sleep(std::time::Duration::from_millis(250));
        // TODO: Load user settings.
        // TODO: Load user projects.
        w_splashscreen
            .close()
            .expect("could not close splashscreen");

        w_main.show().expect("could not show main window");
    });

    Ok(())
}
