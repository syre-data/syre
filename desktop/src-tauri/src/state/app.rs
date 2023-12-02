//! State for the appilcation as a whole.
use crate::settings::{UserAppState, UserSettings};
use std::sync::Mutex;
use thot_core::system::User;
use thot_core::types::ResourceId;

// *****************
// *** App State ***
// *****************

/// App wide state.
#[derive(Default)]
pub struct AppState {
    /// Active user.
    pub user: Mutex<Option<User>>,

    /// Active project.
    pub active_project: Mutex<Option<ResourceId>>,

    /// User settings.
    pub user_settings: Mutex<Option<UserSettings>>,

    /// Application state settings.
    pub user_app_state: Mutex<Option<UserAppState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
