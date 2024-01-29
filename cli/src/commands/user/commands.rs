use super::{AddArgs, EditUserFields};
use crate::Result;
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::system::User;
use syre_core::types::UserId;
use syre_local::system::collections::UserManifest;
use syre_local::system::user_manifest;

/// List all users.
///
/// If verbose, output is of the form `name <email> (id)` with each user on a new line.
/// If not verbose, output is of the form `name <email>` with each user on a new line.
pub fn list(verbose: bool) -> Result {
    let users = match UserManifest::load() {
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

    let users = users.values().map(user_str).collect::<Vec<_>>();
    if users.len() == 0 {
        println!("No users");
        return Ok(());
    }

    println!("{}", users.join("\n"));
    Ok(())
}

pub fn add(user: AddArgs) -> Result {
    let u = User::new(user.email, user.name);
    match user_manifest::add_user(u) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn delete(id: UserId) -> Result {
    let uid = match id {
        UserId::Id(u) => u,
        UserId::Email(email) => {
            let user = match user_manifest::user_by_email(&email) {
                Ok(u) => u,
                Err(err) => return Err(err.into()),
            };

            match user {
                None => return Ok(()),
                Some(u) => u.rid,
            }
        }
    };

    match user_manifest::delete_user(&uid) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn edit(id: UserId, edits: EditUserFields) -> Result {
    let mut user: User = match id {
        UserId::Id(uid) => match user_manifest::user_by_id(&uid) {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(CoreError::Resource(ResourceError::does_not_exist(
                    "user does not exist",
                ))
                .into());
            }
            Err(err) => return Err(err.into()),
        },
        UserId::Email(email) => match user_manifest::user_by_email(&email) {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(CoreError::Resource(ResourceError::DoesNotExist(
                    "user with email `{email}` is not registered".to_string(),
                ))
                .into());
            }
            Err(err) => return Err(err.into()),
        },
    };

    if edits.name.is_some() {
        user.name = edits.name.unwrap();
    }

    if edits.email.is_some() {
        user.email = edits.email.unwrap();
    }

    match user_manifest::update_user(user) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
