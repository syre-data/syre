use super::collections::user_manifest::UserManifest;
use super::settings::user_settings::UserSettings;
use crate::error::{Error, Result, Users as UsersError};
use std::collections::HashMap;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::system::User;
use syre_core::types::ResourceId;
use validator;

// *************
// *** Users ***
// *************

/// Returns a user by the given id if it exists, otherwise returns an error.
pub fn user_by_id(rid: &ResourceId) -> Result<Option<User>> {
    let users = UserManifest::load()?;
    Ok(users.get(&rid).cloned())
}

/// Returns a user by the given email if it exists.
///
/// # Errors
/// + [`UsersError::DuplicateEmail`]: If multiple users are registered with the given email.
pub fn user_by_email(email: &str) -> Result<Option<User>> {
    let users = UserManifest::load()?;
    let users: Vec<&User> = users.values().filter(|user| user.email == email).collect();

    match users.len() {
        0 => Ok(None),
        1 => Ok(Some(users[0].clone())),
        _ => Err(Error::Users(UsersError::DuplicateEmail(email.to_string()))),
    }
}

/// Adds a user to the system settings.
pub fn add_user(user: User) -> Result {
    // validate email
    if !validator::validate_email(&user.email) {
        return Err(UsersError::InvalidEmail(user.email).into());
    }

    let mut users = UserManifest::load_or_default()?;

    // check if email already exists
    let user_count = user_count_by_email(&user.email, &users);
    if user_count > 0 {
        // same email already exists
        return Err(UsersError::DuplicateEmail(user.email.to_string()).into());
    }

    // add user
    users.insert(user.rid.clone(), user);
    users.save()?;
    Ok(())
}

/// Delete a user by id.
pub fn delete_user(rid: &ResourceId) -> Result {
    let mut users = UserManifest::load()?;
    let mut settings = UserSettings::load()?;

    users.remove(&rid);

    // unset as active user, if required
    if settings.active_user.as_ref() == Some(rid) {
        settings.active_user = None;
        settings.save()?;
    }

    users.save()?;
    Ok(())
}

pub fn delete_user_by_email(email: &str) -> Result {
    let Some(user) = user_by_email(email)? else {
        return Ok(());
    };

    delete_user(&user.rid)
}

/// Update the user with the given id.
///  
/// # Errors
/// + [`ResourceError::DoesNotExist`]: A [`User`] with the given id does not exist.
/// + [`UsersError::InvalidEmail`]: The updated email is invalid.
pub fn update_user(user: User) -> Result {
    // validate email
    if !validator::validate_email(&user.email) {
        return Err(UsersError::InvalidEmail(user.email).into());
    }

    let mut users = UserManifest::load()?;
    validate_id_is_present(&user.rid, &users)?;

    users.insert(user.rid.clone(), user);
    users.save()?;
    Ok(())
}

/// Gets the active user.
pub fn get_active_user() -> Result<Option<User>> {
    let user_settings = UserSettings::load_or_default()?;
    let Some(active_user) = user_settings.active_user.as_ref() else {
        return Ok(None);
    };

    user_by_id(&active_user)
}

/// Sets the active user in the system settings.
///
/// # Errors
/// + If the user represented by the id is not registered.
pub fn set_active_user(rid: &ResourceId) -> Result {
    // ensure valid users
    let users = UserManifest::load()?;
    validate_id_is_present(&rid, &users)?;

    // set active user
    let mut settings = UserSettings::load_or_default()?;
    settings.active_user = Some(rid.clone());
    settings.save()?;
    Ok(())
}

/// Sets the active user by email.
///
/// # Errors
/// + If the user represented by the email is not registered.
pub fn set_active_user_by_email(email: &str) -> Result {
    let user = user_by_email(email)?;
    let Some(user) = user else {
        return Err(Error::CoreError(CoreError::Resource(
            ResourceError::does_not_exist("email does not exist"),
        )));
    };

    let mut settings = UserSettings::load()?;
    settings.active_user = Some(user.rid.into());
    settings.save()?;
    Ok(())
}

/// Unsets the active user.
pub fn unset_active_user() -> Result {
    let mut settings = UserSettings::load()?;
    settings.active_user = None;
    settings.save()?;
    Ok(())
}

// *************************
// *** private functions ***
// *************************

/// Returns the number of users with the given email.
fn user_count_by_email(email: &str, users: &UserManifest) -> usize {
    // ensure valid users
    users
        .values()
        .filter(|user| user.email == email)
        .collect::<Vec<&User>>()
        .len()
}

/// Validates that a user exists.
fn validate_id_is_present<V>(rid: &ResourceId, store: &HashMap<ResourceId, V>) -> Result {
    // validate id
    if !store.contains_key(&rid) {
        return Err(
            CoreError::Resource(ResourceError::does_not_exist("`User` does not exist.")).into(),
        );
    }

    Ok(())
}

#[cfg(test)]
#[path = "./user_manifest_test.rs"]
mod user_manifest_test;
