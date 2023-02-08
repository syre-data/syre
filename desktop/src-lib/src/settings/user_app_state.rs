//! Application state for a user.
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct UserAppState {
    /// User that was signed in on last exit.
    pub user: ResourceId,

    /// Open projects during last exit.
    pub open_projects: IndexSet<ResourceId>,

    /// Currently active project.
    pub active_project: Option<ResourceId>,
}

impl UserAppState {
    pub fn new(user: ResourceId) -> Self {
        Self {
            user,
            open_projects: IndexSet::new(),
            active_project: None,
        }
    }
}

impl Default for UserAppState {
    fn default() -> Self {
        Self::new(ResourceId::new())
    }
}

#[cfg(test)]
#[path = "./user_app_state_test.rs"]
mod user_app_state_test;
