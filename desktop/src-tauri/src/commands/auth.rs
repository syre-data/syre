use syre_core::system::User;
use syre_desktop_lib::command::auth::error;
use syre_local::{self as local, error::IoSerde, system::user_manifest};

#[tauri::command]
pub fn register_user(email: String, name: Option<String>) -> Result<User, error::Register> {
    let user = if let Some(name) = name {
        User::with_name(email, name)
    } else {
        User::new(email)
    };

    user_manifest::add_user(user.clone()).map_err(error::Register::AddUser)?;
    user_manifest::set_active_user(user.rid()).map_err(error::Register::SetActiveUser)?;
    Ok(user)
}

#[tauri::command]
pub fn login(email: String) -> Result<User, error::Login> {
    let user = user_manifest::user_by_email(&email).map_err(error::Login::GetUser)?;
    let Some(user) = user else {
        return Err(error::Login::InvalidCredentials);
    };

    user_manifest::set_active_user(user.rid()).map_err(error::Login::SetActiveUser)?;
    Ok(user)
}

#[tauri::command]
pub fn logout() -> Result<(), IoSerde> {
    user_manifest::unset_active_user()
}
