use crate::Result;
use syre_core::types::UserId;
use syre_local::system::user_manifest;

pub fn set_active_user(user: &UserId) -> Result {
    let res = match user {
        UserId::Id(id) => user_manifest::set_active_user(&id),
        UserId::Email(email) => user_manifest::set_active_user_by_email(&email),
    };

    if let Err(err) = res {
        return Err(err.into());
    }

    Ok(())
}

#[cfg(test)]
#[path = "./commands_test.rs"]
mod commands_test;
