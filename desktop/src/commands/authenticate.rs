//! Resources for [`authenticate commands`](thot_desktop_tauri::commands::authenticate).
use crate::common::invoke_result;
use serde::Serialize;
use thot_core::system::User;

pub async fn authenticate_user(email: String) -> Result<Option<User>, String> {
    invoke_result("authenticate_user", UserCredentials { email }).await
}

pub async fn create_user(email: String, name: Option<String>) -> Result<User, String> {
    invoke_result("create_user", CreateUserArgs { email, name }).await
}

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
