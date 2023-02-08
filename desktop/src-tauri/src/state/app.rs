//! State for the appilcation as a whole.
use crate::settings::{UserAppState, UserSettings};
use std::sync::Mutex;
use thot_core::system::User;
use thot_core::types::ResourceId;

// *****************
// *** App State ***
// *****************

/// App wide state.
pub struct AppState {
    /// Active user.
    pub user: Mutex<Option<User>>,

    // /// Loaded projects.
    // pub projects: Mutex<ProjectStore>,
    /// Active project.
    pub active_project: Mutex<Option<ResourceId>>,

    /// User settings.
    pub user_settings: Mutex<Option<UserSettings>>,

    /// Application state settings.
    pub user_app_state: Mutex<Option<UserAppState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user: Default::default(),
            // projects: Mutex::new(ProjectStore::new()),
            active_project: Mutex::new(None),
            user_settings: Mutex::new(None),
            user_app_state: Mutex::new(None),
        }
    }
}

#[cfg(test)]
#[path = "./app_test.rs"]
mod app_test;
