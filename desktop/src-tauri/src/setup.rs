//! Setup functionality for the app.
use crate::state;
use std::thread;
use syre_desktop_lib as lib;
use syre_desktop_resource_db as resource_db;
use syre_project_daemon as project_daemon;
use tauri::{Listener, Manager};
use tauri_plugin_store::StoreExt;
use tauri_plugin_updater::UpdaterExt;

const PROJECT_DAEMON_CONNECTION_ATTEMPTS: usize = 50;
const PROJECT_DAEMON_CONNECTION_DELAY_MS: u64 = 100;
const UPDATE_CHECK_TIMEOUT: u64 = 10; // seconds
const TAURI_SIGNING_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEZBM0MxNjdEMjRBRDc5MTgKUldRWWVhMGtmUlk4K293RjN3MWpUcitrd1l5QVRPbjZxSjRSdmlqRjJDM29GTHcwM0JCUWlGRWEK";

/// Runs setup tasks:
/// 1. Launches `local/project_daemon` if needed.
/// 2. Launches `resource_db`.
/// 3. Launches the update listener.
/// 4. Creates the inital app state.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(app)))]
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(feature = "no-auto-update") {
        let update = tauri::async_runtime::spawn({
            let app = app.handle().clone();
            async move {
                check_for_update(app).await;
            }
        });
        tauri::async_runtime::block_on(update).unwrap();
    }

    setup_project_daemon(app);
    setup_resource_db(app);
    setup_state(app);

    let main = app.get_webview_window("main").unwrap();
    main.listen(crate::project_daemon::FS_EVENT_TOPIC, move |event| {
        #[cfg(feature = "tracing")]
        tracing::debug!(?event);
    });

    Ok(())
}

async fn check_for_update(app: tauri::AppHandle) {
    use lib::settings::app::UpdateChannel;

    let store = app.store(crate::common::DESKTOP_SETTINGS_FILE).unwrap();
    let update_channel = store
        .get("update_channel")
        .map(|channel| serde_json::from_value::<UpdateChannel>(channel).unwrap())
        .unwrap_or_else(|| {
            if cfg!(debug_assertions) {
                UpdateChannel::Debug
            } else {
                UpdateChannel::Stable
            }
        });

    if matches!(update_channel, UpdateChannel::None) {
        return;
    }

    let endpoints = if cfg!(debug_assertions) {
        #[cfg(feature = "tracing")]
        tracing::trace!("checking for updates locally, too");
        vec![
            format!(
                "http://localhost:3030/check?system={{{{target}}}}&arch={{{{arch}}}}&version={{{{current_version}}}}&channel={update_channel}",
            ),
            format!(
                "https://releases.syre.ai/check?system={{{{target}}}}&arch={{{{arch}}}}&version={{{{current_version}}}}&channel={update_channel}",
            ),
        ]
    } else {
        vec![format!(
            "https://releases.syre.ai/check?system={{{{target}}}}&arch={{{{arch}}}}&version={{{{current_version}}}}&channel={update_channel}"
        )]
    };
    let endpoints = endpoints
        .into_iter()
        .map(|uri| uri.parse().unwrap())
        .collect();

    let update = app
        .updater_builder()
        .timeout(std::time::Duration::from_secs(UPDATE_CHECK_TIMEOUT))
        .pubkey(TAURI_SIGNING_PUBLIC_KEY)
        .endpoints(endpoints)
        .unwrap()
        .build()
        .unwrap()
        .check()
        .await;

    let update = match update {
        Ok(update) => update,
        Err(err) => {
            #[cfg(feature = "tracing")]
            tracing::warn!("could not retrieve update: {err:?}");
            return;
        }
    };

    if let Some(update) = update {
        #[cfg(feature = "tracing")]
        tracing::trace!("update available");
        let mut downloaded = 0;
        let response = update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    #[cfg(feature = "tracing")]
                    tracing::trace!("downloaded {downloaded} of {content_length:?}");
                },
                || {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("download finished");
                },
            )
            .await;

        if let Err(err) = response {
            #[cfg(feature = "tracing")]
            tracing::warn!("could not download or install new version: {err:?}");
        } else {
            #[cfg(feature = "tracing")]
            tracing::trace!("update installed, restarting app");
            app.restart();
        }
    } else {
        #[cfg(feature = "tracing")]
        tracing::trace!("no update available");
    }
}

fn setup_project_daemon(app: &mut tauri::App) {
    if let Some((_rx, _child)) = crate::project_daemon::start_project_daemon_if_needed(app.handle())
    {
        #[cfg(feature = "tracing")]
        tracing::trace!("initializing project daemon");
        let mut attempt = 0;
        while !project_daemon::Client::server_available() {
            attempt += 1;
            if attempt > PROJECT_DAEMON_CONNECTION_ATTEMPTS {
                panic!("could not connect to project daemon");
            }

            std::thread::sleep(std::time::Duration::from_millis(
                PROJECT_DAEMON_CONNECTION_DELAY_MS,
            ));
        }

        #[cfg(feature = "tracing")]
        tracing::debug!("initialized project daemon");
    } else {
        #[cfg(feature = "tracing")]
        tracing::debug!("project daemon already running");
    };

    let actor = crate::project_daemon::actor::Builder::new(app.handle().clone());
    std::thread::Builder::new()
        .name("syre desktop project daemon event listener".to_string())
        .spawn(move || actor.run())
        .unwrap();

    // REMOVE
    use tauri::Emitter;
    std::thread::spawn({
        let app = app.handle().clone();
        move || {
            let mut i = 0;
            loop {
                let path: std::path::PathBuf = format!("/test/{i}").into();
                app.emit(
                    lib::event::topic::PROJECT_MANIFEST,
                    vec![lib::Event::new(
                        lib::event::EventKind::ProjectManifest(lib::event::ProjectManifest::Added(
                            vec![(
                                path.clone(),
                                project_daemon::state::ProjectData {
                                    properties: project_daemon::state::DataResource::Ok(
                                        syre_core::project::Project::new(format!("project {i}")),
                                    ),
                                    settings: project_daemon::state::DataResource::Ok(
                                        syre_local::project::config::Settings::new(),
                                    ),
                                    analyses: project_daemon::state::DataResource::Ok(vec![]),
                                },
                            )],
                        )),
                        uuid::Uuid::new_v4(),
                    )],
                )
                .unwrap();
                tracing::debug!("SENT {i}");
                i += 1;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    });
}

fn setup_resource_db(app: &mut tauri::App) {
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

    let db = resource_db::Builder::new(command_rx);
    thread::Builder::new()
        .name("syre desktop resource database".to_string())
        .spawn(move || db.run().unwrap())
        .unwrap();

    let client = resource_db::Client::new(command_tx);
    assert!(app.manage(client));
}

fn setup_state(app: &mut tauri::App) {
    let project_client = app.state::<project_daemon::Client>();
    let state = crate::State::new();
    if let project_daemon::state::ConfigState::Ok(local_config) =
        project_client.state().local_config().unwrap()
    {
        if let Some(user) = local_config.user {
            let projects = state::load_user_state(&project_client, &user);
            let _ = state
                .user()
                .lock()
                .unwrap()
                .insert(state::User::new(user, projects));
        }
    }
    assert!(app.manage(state));
}
