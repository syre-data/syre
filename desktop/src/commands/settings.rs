//! Command functionality for settings.
use super::common::ResourceIdArgs;
use crate::common::invoke_result;
use serde::Serialize;
use thot_core::types::ResourceId;
use thot_desktop_lib::settings::{UserAppState, UserSettings};
use thot_local::error::IoSerde;

pub async fn load_user_app_state(user: ResourceId) -> Result<UserAppState, IoSerde> {
    invoke_result("load_user_app_state", ResourceIdArgs { rid: user }).await
}

pub async fn load_user_settings(user: ResourceId) -> Result<UserSettings, IoSerde> {
    invoke_result("load_user_settings", ResourceIdArgs { rid: user }).await
}

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
