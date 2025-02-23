//! Setup functionality for the app.
use crate::state;
use std::thread;
use syre_project_watcher as project_watcher;
use syre_resource_db as resource_db;
use tauri::{Listener, Manager};
use tauri_plugin_updater::UpdaterExt;

const PROJECT_WATCHER_CONNECTION_ATTEMPTS: usize = 50;
const PROJECT_WATCHER_CONNECTION_DELAY_MS: u64 = 100;
const UPDATE_CHECK_TIMEOUT: u64 = 30; // in seconds
const TAURI_SIGNING_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEZBM0MxNjdEMjRBRDc5MTgKUldRWWVhMGtmUlk4K293RjN3MWpUcitrd1l5QVRPbjZxSjRSdmlqRjJDM29GTHcwM0JCUWlGRWEK";

/// Runs setup tasks:
/// 1. Launches `local/project_watcher` if needed.
/// 2. Launches `resource_db`.
/// 3. Launches the update listener.
/// 4. Creates the inital app state.
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

    setup_project_watcher(app);
    setup_resource_db(app);
    setup_state(app);

    let main = app.get_webview_window("main").unwrap();
    main.listen(crate::project_watcher::FS_EVENT_TOPIC, move |event| {
        tracing::debug!(?event);
    });

    Ok(())
}

async fn check_for_update(app: tauri::AppHandle) {
    let endpoints = if cfg!(debug_assertions) {
        tracing::trace!("checking for updates locally");
        vec![
            "http://localhost:3030/check?system={{target}}&arch={{arch}}&version={{current_version}}&debug=true",
            "https://releases.syre.ai/check?system={{target}}&arch={{arch}}&version={{current_version}}&debug=true",
        ]
    } else {
        vec!["https://releases.syre.ai/check?system={{target}}&arch={{arch}}&version={{current_version}}"]
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
            tracing::error!("could not retrieve update: {err:?}");
            return;
        }
    };

    if let Some(update) = update {
        let mut downloaded = 0;

        let response = update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    tracing::trace!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    tracing::trace!("download finished");
                },
            )
            .await;

        if let Err(err) = response {
            tracing::error!("could not download or install new version: {err:?}");
        } else {
            tracing::trace!("update installed, restarting app");
            app.restart();
        }
    }
}

fn setup_project_watcher(app: &mut tauri::App) {
    if let Some((_rx, _child)) =
        crate::project_watcher::start_project_watcher_if_needed(app.handle())
    {
        tracing::trace!("initializing project watcher");
        let mut attempt = 0;
        while !project_watcher::Client::server_available() {
            attempt += 1;
            if attempt > PROJECT_WATCHER_CONNECTION_ATTEMPTS {
                panic!("could not connect to project watcher");
            }

            std::thread::sleep(std::time::Duration::from_millis(
                PROJECT_WATCHER_CONNECTION_DELAY_MS,
            ));
        }

        tracing::debug!("initialized project watcher");
    } else {
        tracing::debug!("project watcher already running");
    };

    let actor = crate::project_watcher::actor::Builder::new(app.handle().clone());
    std::thread::Builder::new()
        .name("syre desktop project watcher event listener".to_string())
        .spawn(move || actor.run())
        .unwrap();
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
    let project_client = app.state::<project_watcher::Client>();
    let state = crate::State::new();
    if let project_watcher::state::ConfigState::Ok(local_config) =
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
