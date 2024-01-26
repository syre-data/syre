//! Functionality to interact with [`UserAppState`] settings.
use crate::error::{DesktopSettings as DesktopSettingsError, Result};
use crate::settings::UserAppState;
use crate::state::AppState;
use std::result::Result as StdResult;
use tauri::State;
use thot_core::types::ResourceId;
use thot_desktop_lib::settings::{HasUser, UserAppState as DesktopUserAppState};
use thot_local::error::IoSerde;

/// Loads a user's [`UserAppState`](DesktopUserAppState) settings.
/// Maintains control of the settings file.
#[tauri::command]
pub fn load_user_app_state(
    app_state: State<AppState>,
    rid: ResourceId,
) -> StdResult<DesktopUserAppState, IoSerde> {
    let mut state = app_state
        .user_app_state
        .lock()
        .expect("could not lock `AppState.user_app_state`");

    if let Some(state) = state.as_ref() {
        // user state loaded
        if state.user() == &rid {
            // user state for user already loaded
            return Ok((*state).clone().into());
        }
    }

    let user_app_state = UserAppState::load_or_new(&rid)?;
    *state = Some(user_app_state.clone());

    Ok(user_app_state.into())
}

/// Gets the current [`UserAppState`].
#[tauri::command]
pub fn get_user_app_state(app_state: State<AppState>) -> Option<DesktopUserAppState> {
    let state = app_state
        .user_app_state
        .lock()
        .expect("could not lock `UserAppState`");

    state.as_ref().map(|state| (*state).clone().into())
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
        return Err(DesktopSettingsError::InvalidUpdate(
            "`AppState.user_app_state` not loaded".to_string(),
        )
        .into());
    };

    user_app_state.update(state)?;
    user_app_state.save()?;
    Ok(())
}
