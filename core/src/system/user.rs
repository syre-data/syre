//! User.
use crate::types::ResourceId;
use chrono::prelude::*;
use has_id::HasId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

#[cfg(feature = "yew")]
use yew::prelude::*;

// ************
// *** User ***
// ************

/// Represents a User.
#[cfg_attr(feature = "yew", derive(Properties))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, HasIdSerde))]
#[derive(HasId, Clone, Debug, PartialEq)]
pub struct User {
    #[id]
    pub rid: ResourceId,
    pub created: DateTime<Utc>,
    pub email: String,
    pub name: Option<String>,
}

impl User {
    pub fn new(email: String, name: Option<String>) -> User {
        User {
            rid: ResourceId::new(),
            created: Utc::now(),
            name,
            email,
        }
    }
}

#[cfg(test)]
#[path = "./user_test.rs"]
mod user_test;
