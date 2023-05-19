#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(path_file_prefix)]
mod commands;
mod common;
mod db;
mod error;
mod identifier;
mod settings;
mod setup;
mod state;
mod ui;

use commands::*;
use std::io;
use tauri::RunEvent;
use thot_local_database::client::Client as DbClient;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{Layer, Registry};

use ui::{handle_menu_event, handle_system_tray_event, main_menu, system_tray};

const LOG_PREFIX: &str = "desktop.log";
const MAX_LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    // logging setup
    let config_dir = common::config_dir_path().expect("could not get config dir path");
    let file_logger = tracing_appender::rolling::daily(config_dir, LOG_PREFIX);
    let (file_logger, _log_guard) = tracing_appender::non_blocking(file_logger);
    let file_logger = fmt::layer()
        .with_writer(file_logger)
        .with_timer(UtcTime::rfc_3339())
        .json()
        // .pretty()
        .with_filter(MAX_LOG_LEVEL);

    let console_logger = fmt::layer()
        .with_writer(io::stdout)
        .with_timer(UtcTime::rfc_3339())
        .pretty()
        .with_filter(MAX_LOG_LEVEL);

    let subscriber = Registry::default().with(console_logger).with(file_logger);

    tracing::subscriber::set_global_default(subscriber).expect("could not create logger");

    // check for database, create if needed
    let _db_handler = db::functions::verify_database();

    // create app
    let app = tauri::Builder::default()
        .system_tray(system_tray())
        .on_system_tray_event(|app, event| handle_system_tray_event(app, event))
        .menu(main_menu())
        .on_menu_event(handle_menu_event)
        .manage(state::AppState::new())
        .manage(DbClient::new())
        .invoke_handler(tauri::generate_handler![
            // authenticate
            authenticate_user,
            // common
            get_directory,
            open_file,
            // settings
            get_user_app_state,
            get_user_settings,
            load_user_app_state,
            load_user_settings,
            update_user_app_state,
            update_user_settings,
            // user
            create_user,
            set_active_user,
            get_active_user,
            unset_active_user,
            // project
            add_project,
            get_project_path,
            init_project,
            set_active_project,
            load_project,
            load_user_projects,
            get_project,
            // new_project, @todo: Possibly remove.
            update_project,
            analyze,
            // graph
            init_project_graph,
            load_project_graph,
            // container
            add_assets,
            add_asset_windows,
            get_container,
            get_container_path,
            new_child,
            update_container_properties,
            update_container_script_associations,
            duplicate_container_tree,
            remove_container_tree,
            // asset
            get_assets,
            update_asset_properties,
            remove_asset,
            // script
            get_project_scripts,
            add_script,
            remove_script,
        ])
        .setup(setup::setup)
        .build(tauri::generate_context!())
        .expect("could not build app");

    app.run(move |_app, event| match event {
        RunEvent::ExitRequested { api, .. } => {
            // @todo: Appears that `database` process is killed automatically
            // when parent is killed. May have to manually kill if detached.
            // if let Some((_rx_database, proc_database)) = db_handler {
            //     proc_database
            //         .kill()
            //         .expect("could not kill `database` process");
            // }
        }
        _ => {}
    });
}
