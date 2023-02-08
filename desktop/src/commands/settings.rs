//! Command functionality for settings.
use serde::Serialize;
use thot_desktop_lib::settings::{UserAppState, UserSettings};

/// Argument for commands requiring only a [`UserAppState`] named `state`.
#[derive(Serialize, Debug)]
pub struct UserAppStateArgs {
    pub state: UserAppState,
}

/// Argument for commands requiring only a [`UserSettings`] named `settings`.
#[derive(Serialize, Debug)]
pub struct UserSettingsArgs {
    pub settings: UserSettings,
}

#[cfg(test)]
#[path = "./settings_test.rs"]
mod settings_test;
