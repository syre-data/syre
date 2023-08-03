//! Functionality to interact with [`UserAppState`] settings.
use crate::error::{DesktopSettingsError, Result};
use crate::settings::{loader::Loader, UserAppState};
use crate::state::AppState;
use settings_manager::Settings;
use tauri::State;
use thot_core::types::ResourceId;
use thot_desktop_lib::settings::{HasUser, UserAppState as DesktopUserAppState};

/// Loads a user's [`UserAppState`](DesktopUserAppState) settings.
/// Maintains control of the settings file.
#[tauri::command]
pub fn load_user_app_state(
    app_state: State<AppState>,
    rid: ResourceId,
) -> Result<DesktopUserAppState> {
    let mut state = app_state
        .user_app_state
        .lock()
        .expect("could not lock `AppState.user_app_state`");

    if let Some(state) = state.as_ref() {
        // user state loaded
        if state.user() == &rid {
            // user state for user already loaded
            return Ok((*state).clone());
        }
    }

    let user_state: UserAppState = Loader::load_or_create_with::<UserAppState>(&rid)?.into();
    let desktop_state = user_state.clone().into();
    *state = Some(user_state);

    Ok(desktop_state)
}

/// Gets the current [`UserAppState`].
#[tauri::command]
pub fn get_user_app_state(app_state: State<AppState>) -> Option<DesktopUserAppState> {
    let state = app_state
        .user_app_state
        .lock()
        .expect("could not lock `UserAppState`");

    state.as_ref().map(|settings| (*settings).clone())
}

/// Updates a user's [`UserAppState`](DesktopUserAppState) settings.
///
/// # Errors
/// + [`DesktopSettings::InvalidUpdate`] if a [`UserAppState`] is not loaded.
#[tauri::command]
pub fn update_user_app_state(app_state: State<AppState>, state: DesktopUserAppState) -> Result {
    // verify correct user.

    let mut user_app_state = app_state
        .user_app_state
        .lock()
        .expect("could not lock `user_app_state`");

    let Some(user_app_state) = user_app_state.as_mut() else {
        // settings not loaded
        return Err(DesktopSettingsError::InvalidUpdate("`AppState.user_app_state` not loaded".to_string()).into());
    };

    user_app_state.update(state)?;
    user_app_state.save()?;
    Ok(())
}
