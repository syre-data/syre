// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use syre_desktop::{
    commands::{analyses, asset, auth, bulk, common, container, fs, graph, project, user},
    setup,
};

fn main() {
    let builder = tauri::Builder::default();

    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let builder = builder.plugin(tauri_plugin_devtools::init());
    #[cfg(not(debug_assertions))]
    logging::enable();

    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(syre_local_database::Client::new())
        .invoke_handler(tauri::generate_handler![
            auth::register_user,
            auth::login,
            auth::logout,
            user::active_user,
            user::user_count,
            user::user_projects,
            project::create_project,
            project::initialize_project,
            project::import_project,
            project::deregister_project,
            project::project_resources,
            project::project_analysis_remove,
            project::project_properties_update,
            project::analyze_project,
            project::delete_project,
            analyses::add_scripts,
            graph::create_child_container,
            graph::add_file_system_resources,
            graph::container_duplicate,
            graph::container_trash,
            container::container_rename,
            container::container_rename_bulk,
            container::container_properties_update,
            container::container_properties_update_bulk,
            container::container_analysis_associations_update,
            asset::asset_properties_update,
            asset::asset_properties_update_bulk,
            asset::asset_remove_file,
            bulk::properties_update_bulk,
            fs::pick_folder,
            fs::pick_folder_with_location,
            common::open_file,
            common::target_os,
        ])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(debug_assertions))]
mod logging {
    use tracing_subscriber::{
        fmt::{self, time::UtcTime},
        prelude::*,
        Layer, Registry,
    };

    const LOG_PREFIX: &str = "desktop.log";

    pub fn enable() {
        let config_dir = syre_local::system::common::config_dir_path().unwrap();
        let file_logger = tracing_appender::rolling::daily(config_dir, LOG_PREFIX);
        let (file_logger, _log_guard) = tracing_appender::non_blocking(file_logger);
        let file_logger = fmt::layer()
            .with_writer(file_logger)
            .with_timer(UtcTime::rfc_3339())
            .json()
            .with_filter(MAX_LOG_LEVEL);

        let console_logger = fmt::layer()
            .with_writer(std::io::stdout)
            .with_timer(UtcTime::rfc_3339())
            .pretty()
            .with_filter(MAX_LOG_LEVEL);

        let subscriber = Registry::default().with(console_logger).with(file_logger);
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}
