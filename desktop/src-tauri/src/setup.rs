//! Startup functionality.
use tauri::{App, Manager};

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // get windows
    let w_splashscreen = app
        .get_window("splashscreen")
        .expect("could not get splashscreen");

    let w_main = app.get_window("main").expect("could not get main window");

    // run init in new task
    tauri::async_runtime::spawn(async move {
        // @todo: Load user settings.
        // @todo: Load user projects.
        std::thread::sleep(std::time::Duration::from_millis(500));
        w_splashscreen
            .close()
            .expect("could not close splashscreen");

        w_main.show().expect("could not show main window");
    });

    Ok(())
}

#[cfg(test)]
#[path = "./setup_test.rs"]
mod setup_test;
