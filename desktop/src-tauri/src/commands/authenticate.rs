//! Authentication functionality.
use crate::error::Result;
use thot_core::system::User;
use thot_local::system::users;

/// Authenticate a user's credentials.
#[tauri::command]
pub fn authenticate_user(email: &str) -> Result<Option<User>> {
    let user = users::user_by_email(email)?;
    Ok(user)
}

/// Create a new user account.
#[tauri::command]
pub fn create_user(email: String, name: Option<String>) -> Result<User> {
    let user = User::new(email, name);
    users::add_user(user.clone())?;

    Ok(user)
}
