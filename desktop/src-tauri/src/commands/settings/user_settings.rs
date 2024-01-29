//! Commands to interact with [`UserSettings`].
use crate::error::{DesktopSettings as DesktopSettingsError, Result};
use crate::settings::UserSettings as AppUserSettings;
use crate::state::AppState;
use std::result::Result as StdResult;
use syre_core::types::ResourceId;
use syre_desktop_lib::settings::{HasUser, UserSettings as DesktopUserSettings};
use syre_local::error::IoSerde;
use tauri::State;

/// Loads a user's [`UserSettings`](DesktopUserSettings) from file and stores them.
#[tracing::instrument(skip(app_state))]
#[tauri::command]
pub fn load_user_settings(
    app_state: State<AppState>,
    rid: ResourceId,
) -> StdResult<DesktopUserSettings, IoSerde> {
    let mut settings = app_state
        .user_settings
        .lock()
        .expect("could not lock `AppState.user_settings`");

    if let Some(settings) = settings.as_ref() {
        // user settings loaded
        if settings.user() == &rid {
            // user settings for user already loaded
            let desktop_settings: DesktopUserSettings = (*settings).clone();
            return Ok(desktop_settings);
        }
    }

    let user_settings = AppUserSettings::load_or_new(&rid)?;
    let desktop_settings: DesktopUserSettings = user_settings.clone().into();
    *settings = Some(user_settings);

    Ok(desktop_settings)
}

/// Gets the currently loaded [`UserSettings`](DersktopUserSettings).
#[tauri::command]
pub fn get_user_settings(app_state: State<AppState>) -> Option<DesktopUserSettings> {
    let settings = app_state
        .user_settings
        .lock()
        .expect("could not lock `AppState.user_settings`");

    settings.as_ref().map(|settings| (*settings).clone())
}

/// Update a user's [`UserSettings`](DesktopUserSettings).
#[tauri::command]
pub fn update_user_settings(app_state: State<AppState>, settings: DesktopUserSettings) -> Result {
    let mut user_settings = app_state
        .user_settings
        .lock()
        .expect("could not lock `AppState.user_settings`");

    let Some(user_settings) = user_settings.as_mut() else {
        return Err(DesktopSettingsError::InvalidUpdate(
            "`AppState.user_settings` not loaded".to_string(),
        )
        .into());
    };

    user_settings.update(settings)?;
    user_settings.save()?;
    Ok(())
}
