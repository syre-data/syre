//! Resources for [`authenticate commands`](thot_desktop_tauri::commands::authenticate).
use serde::Serialize;

/// User credentials for authentication.
#[derive(Serialize)]
pub struct UserCredentials {
    pub email: String,
}

/// User info for creating account.
#[derive(Serialize)]
pub struct CreateUserArgs {
    pub name: Option<String>,
    pub email: String,
}
