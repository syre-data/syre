use crate::settings;
use std::{io, path::PathBuf};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_local as local;
use tauri_plugin_store::StoreExt;

/// Retrieve the desktop settings for the active user.
/// If none are set, uses default.
#[tauri::command]
pub fn user_settings(state: tauri::State<crate::State>) -> Option<lib::settings::User> {
    let user = state.user();
    let user = user.lock().unwrap();
    let Some(user) = user.as_ref() else {
        return None;
    };

    let settings = settings::User::load(user.rid()).replace_not_found_with_default();
    Some(settings.into())
}

/// Update the desktop settings for the active user.
#[tauri::command]
pub fn user_settings_desktop_update(
    state: tauri::State<crate::State>,
    user: ResourceId,
    update: lib::settings::user::Desktop,
) -> Result<(), local::error::IoSerde> {
    let state_user = state.user();
    let state_user = state_user.lock().unwrap();
    let Some(ref state_user) = *state_user else {
        panic!("invalid state");
    };
    assert_eq!(user, *state_user.rid());

    let mut settings = settings::user::Desktop::load(&user)?;
    settings.input_debounce_ms = update.input_debounce_ms;
    settings.save(&user).map_err(|err| err.kind().into())
}

/// Update the runner settings for the active user.
#[tauri::command]
pub fn user_settings_runner_update(
    state: tauri::State<crate::State>,
    user: ResourceId,
    update: lib::settings::user::Runner,
) -> Result<(), lib::command::error::IoErrorKind> {
    let state_user = state.user();
    let state_user = state_user.lock().unwrap();
    let Some(ref state_user) = *state_user else {
        panic!("invalid state");
    };
    assert_eq!(user, *state_user.rid());

    settings::user::Runner::save(&user, update).map_err(|err| err.kind().into())
}

/// Update the analysis settings for the active user.
#[tauri::command]
pub fn user_settings_analysis_update(
    state: tauri::State<crate::State>,
    user: ResourceId,
    update: lib::settings::user::Analysis,
) -> Result<(), local::error::IoSerde> {
    let state_user = state.user();
    let state_user = state_user.lock().unwrap();
    let Some(ref state_user) = *state_user else {
        panic!("invalid state");
    };
    assert_eq!(user, *state_user.rid());

    let mut settings = settings::user::Desktop::load(&user)?;
    settings.disable_analysis_after = update.disable_analysis_after;
    settings.save(&user).map_err(|err| err.kind().into())
}

/// Retrieve the project settings.
/// If none are set, uses default.
#[tauri::command]
pub fn project_settings(project: PathBuf) -> lib::settings::Project {
    settings::Project::load(&project)
        .replace_not_found_with_default()
        .into()
}

/// Update the desktop settings for the project.
#[tauri::command]
pub fn project_settings_desktop_update(
    project: PathBuf,
    update: lib::settings::project::Desktop,
) -> Result<(), local::error::IoSerde> {
    let mut settings = settings::project::Desktop::load(&project).or_else(|err| {
        if let local::error::IoSerde::Io(err) = err {
            if matches!(err, io::ErrorKind::NotFound) {
                return Ok(settings::project::Desktop::default());
            }
        }

        Err(err)
    })?;
    settings.asset_drag_drop_kind = update.asset_drag_drop_kind;
    settings::project::Desktop::save(&project, settings).map_err(|err| err.kind().into())
}

/// Update the runner settings for the project.
#[tauri::command]
pub fn project_settings_runner_update(
    project: PathBuf,
    update: lib::settings::project::Runner,
) -> Result<(), lib::command::error::IoErrorKind> {
    settings::project::Runner::save(&project, update).map_err(|err| err.into())
}

/// Update the desktop settings for the project.
#[tauri::command]
pub fn project_settings_analysis_update(
    project: PathBuf,
    update: lib::settings::project::Analysis,
) -> Result<(), local::error::IoSerde> {
    let mut settings = settings::project::Desktop::load(&project).or_else(|err| {
        if let local::error::IoSerde::Io(err) = err {
            if matches!(err, io::ErrorKind::NotFound) {
                return Ok(settings::project::Desktop::default());
            }
        }

        Err(err)
    })?;

    settings.disable_analysis_after = update.disable_analysis_after;
    settings::project::Desktop::save(&project, settings).map_err(|err| err.kind().into())
}

#[tauri::command]
pub fn app_settings(app: tauri::AppHandle) -> lib::settings::App {
    let store = app.store(crate::common::DESKTOP_SETTINGS_FILE).unwrap();
    let update_channel = store
        .get("update_channel")
        .map(|channel| serde_json::from_value(channel).unwrap())
        .unwrap_or(lib::settings::app::UpdateChannel::default());

    lib::settings::App { update_channel }
}

#[tauri::command]
pub fn app_settings_update(
    app: tauri::AppHandle,
    update: lib::settings::App,
) -> Result<(), lib::command::error::IoErrorKind> {
    let store = app.store(crate::common::DESKTOP_SETTINGS_FILE).unwrap();
    store.set(
        "update_channel",
        serde_json::to_value(update.update_channel).unwrap(),
    );
    let result = store.save();
    if let Err(err) = result.as_ref() {
        tracing::error!("could not save desktop settings: {err:?}");
    }

    result.map_err(|err| match err {
        tauri_plugin_store::Error::Io(err) => err.into(),
        _ => panic!("{err:?}"),
    })
}
