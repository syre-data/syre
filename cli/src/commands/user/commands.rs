use super::{AddArgs, EditUserFields};
use crate::result::Result;
use thot_core::result::{Error as CoreError, ResourceError};
use settings_manager::{ListSetting, SystemSettings};
use thot_core::system::User;
use thot_core::types::UserId;
use thot_local::system::collections::Users;
use thot_local::system::users;

/// List all users.
///
/// If verbose, output is of the form `name <email> (id)` with each user on a new line.
/// If not verbose, output is of the form `name <email>` with each user on a new line.
pub fn list(verbose: bool) -> Result {
    let mut users = match Users::load() {
        Ok(sets) => sets,
        Err(err) => panic!("Something went wrong: {:?}", err),
    };

    let user_str = match verbose {
        true => |user: &User| match &user.name {
            None => format!("{} ({})", user.email, user.rid),
            Some(name) => format!("{} <{}> ({})", user.email, name, user.rid),
        },
        false => |user: &User| match &user.name {
            None => format!("{}", user.email),
            Some(name) => format!("{} <{}>", user.email, name),
        },
    };

    let users: Vec<String> = users.items().iter().map(user_str).collect();

    let mut out = String::new();
    if users.len() == 0 {
        out.push_str("No users");
    } else {
        for user in users {
            out.push_str(user.as_str());
            out.push('\n');
        }

        out.pop(); // remove last new line
    }

    println!("{}", out);
    Ok(())
}

pub fn add(user: AddArgs) -> Result {
    let u = User::new(user.email, user.name);
    match users::add_user(u) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn delete(id: UserId) -> Result {
    let uid = match id {
        UserId::Id(u) => u,
        UserId::Email(email) => {
            let user = match users::user_by_email(&email) {
                Ok(u) => u,
                Err(err) => return Err(err.into()),
            };

            match user {
                None => return Ok(()),
                Some(u) => u.rid,
            }
        }
    };

    match users::delete_user(&uid) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn edit(id: UserId, edits: EditUserFields) -> Result {
    let mut user: User = match id {
        UserId::Id(uid) => match users::user_by_id(&uid) {
            Ok(u) => u,
            Err(err) => return Err(err.into()),
        },
        UserId::Email(email) => match users::user_by_email(&email) {
            Ok(u) => match u {
                None => return Err(CoreError::ResourceError(ResourceError::DoesNotExist("user with email `{email}` is not registered".to_string())).into()),
                Some(u) => u,
            },
            Err(err) => return Err(err.into()),
        },
    };

    if edits.name.is_some() {
        user.name = edits.name.unwrap();
    }

    if edits.email.is_some() {
        user.email = edits.email.unwrap();
    }

    match users::update_user(user) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
#[path = "./commands_test.rs"]
mod commands_test;
