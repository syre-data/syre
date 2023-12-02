use crate::Result;
use thot_core::types::UserId;
use thot_local::system::users;

pub fn set_active_user(user: &UserId) -> Result {
    let res = match user {
        UserId::Id(id) => users::set_active_user(&id),
        UserId::Email(email) => users::set_active_user_by_email(&email),
    };

    if let Err(err) = res {
        return Err(err.into());
    }

    Ok(())
}

#[cfg(test)]
#[path = "./commands_test.rs"]
mod commands_test;
