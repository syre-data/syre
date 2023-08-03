use super::resource_id::ResourceId;
use super::user_id::UserId;
use std::cmp::PartialEq;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Creator of a resource.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq)]
pub enum Creator {
    User(Option<UserId>),
    Script(ResourceId),
}

impl Default for Creator {
    fn default() -> Self {
        Creator::User(None)
    }
}

impl PartialEq for Creator {
    fn eq(&self, other: &Creator) -> bool {
        match (self, other) {
            (Creator::User(me), Creator::User(you)) => me == you,
            (Creator::Script(me), Creator::Script(you)) => me == you,
            _ => false,
        }
    }
}
