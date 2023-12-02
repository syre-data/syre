//! Interaction functionality with a [`Database`].
use tauri::api::process::{Command, CommandChild, CommandEvent};
use tauri::async_runtime::Receiver;
use thot_local_database::Client as DbClient;

pub fn verify_database() -> Option<(Receiver<CommandEvent>, CommandChild)> {
    // try to connect to database
    if DbClient::server_available() {
        return None;
    }

    // database not running
    // create one
    let handler = init_database();
    Some(handler)
}

// Important On macOS m1, not dropping the `Receiver` (part of the _db_handler), causes ZMQ issues.
/// Initializes a [`Database`] as a sidecar process.
fn init_database() -> (Receiver<CommandEvent>, CommandChild) {
    Command::new_sidecar("thot-local-database")
        .expect("failed to create `thot-local-database` binary command")
        .spawn()
        .expect("failed to spawn sidecar")
}
