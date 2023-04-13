//! Application state for a user.
use super::HasUser;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct UserAppState {
    /// User that was signed in on last exit.
    user: ResourceId,

    /// Open projects during last exit.
    pub open_projects: IndexSet<ResourceId>,

    /// Currently active project.
    pub active_project: Option<ResourceId>,
}

impl HasUser for UserAppState {
    fn new(user: ResourceId) -> Self {
        Self {
            user,
            open_projects: IndexSet::new(),
            active_project: None,
        }
    }

    fn user(&self) -> &ResourceId {
        &self.user
    }
}

#[cfg(test)]
#[path = "./user_app_state_test.rs"]
mod user_app_state_test;
