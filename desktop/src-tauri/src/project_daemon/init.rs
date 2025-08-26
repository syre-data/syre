//! Initialization functionality with a [`Database`].
use syre_project_daemon::Client as DaemonClient;
use tauri::async_runtime::Receiver;
use tauri_plugin_shell::{
    ShellExt,
    process::{CommandChild, CommandEvent},
};

/// Init
pub fn start_project_daemon_if_needed(
    app: &tauri::AppHandle,
) -> Option<(Receiver<CommandEvent>, CommandChild)> {
    // try to connect to database
    if DaemonClient::server_available() {
        return None;
    }

    // database not running
    // create one
    let handler = init_project_daemon(app);
    Some(handler)
}

// IMPORTANT: On macOS m1, not dropping the `Receiver` (part of the _db_handler), causes ZMQ issues.
/// Initializes a [`Database`] as a sidecar process.
fn init_project_daemon(app: &tauri::AppHandle) -> (Receiver<CommandEvent>, CommandChild) {
    app.shell()
        .sidecar("syre-project-daemon")
        .expect("failed to create `syre-project-daemon` binary command")
        .spawn()
        .expect("failed to spawn sidecar")
}
