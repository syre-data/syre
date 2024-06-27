//! Authentication functionality.
use crate::error::Result;
use syre_core::system::User;
use syre_local::system::user_manifest;

/// Authenticate a user's credentials.
#[tauri::command]
pub fn authenticate_user(email: &str) -> Result<Option<User>> {
    let user = user_manifest::user_by_email(email)?;
    Ok(user)
}

/// Create a new user account.
#[tauri::command]
pub fn create_user(email: String, name: Option<String>) -> Result<User> {
    let user = User::new(email, name);
    user_manifest::add_user(user.clone())?;

    Ok(user)
}
