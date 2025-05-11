// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use syre_desktop::{
    commands::{
        analyses, asset, auth, common, container, fs, graph, mixed_bulk, project, query, settings,
        user,
    },
    setup, state,
};

fn main() {
    let _log_guard = logging::enable();
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .manage(syre_project_watcher::Client::new())
        .manage(state::new_slice(Option::<state::AnalyzerAction>::None))
        .invoke_handler(tauri::generate_handler![
            analyses::project_add_analyses,
            analyses::analysis_toggle_associations,
            asset::asset_properties_update_bulk,
            asset::asset_properties_update,
            asset::asset_remove_file,
            auth::login,
            auth::logout,
            auth::register_user,
            common::file_size,
            common::open_file,
            common::target_os,
            container::container_analysis_associations_update_bulk,
            container::container_analysis_associations_update,
            container::container_properties_update_bulk,
            container::container_properties_update,
            container::container_rename_bulk,
            container::container_rename,
            container::remove_flag,
            container::remove_all_flags,
            fs::pick_file_with_location,
            fs::pick_folder_with_location,
            fs::pick_folder,
            graph::add_file_system_resources,
            graph::container_duplicate,
            graph::container_trash,
            graph::create_child_container,
            mixed_bulk::properties_update_bulk_mixed,
            project::trigger_analysis,
            project::cancel_analysis,
            project::kill_analysis,
            project::create_project,
            project::delete_project,
            project::deregister_project,
            project::duplicate_project,
            project::import_project,
            project::initialize_project,
            project::project_analysis_remove,
            project::project_properties_update,
            project::project_resources,
            settings::app_settings,
            settings::app_settings_update,
            settings::user_settings,
            settings::user_settings_desktop_update,
            settings::user_settings_runner_update,
            settings::user_settings_analysis_update,
            settings::project_settings,
            settings::project_settings_desktop_update,
            settings::project_settings_runner_update,
            settings::project_settings_analysis_update,
            user::active_user,
            user::user_count,
            user::user_projects,
            query::search_project,
            query::search_project_assets,
        ])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod logging {
    use syre_desktop as desktop;
    use tracing_subscriber::{
        Registry, filter,
        fmt::{self, time},
        prelude::*,
    };

    const LOG_PREFIX: &str = "desktop.log";

    #[cfg(debug_assertions)]
    const SYRE_LOG_LEVEL_FILE: tracing::Level = tracing::Level::DEBUG;
    #[cfg(not(debug_assertions))]
    const SYRE_LOG_LEVEL_FILE: tracing::Level = tracing::Level::ERROR;

    pub fn enable() -> tracing_appender::non_blocking::WorkerGuard {
        let config_dir = desktop::common::config_dir_path().unwrap();
        let file_filter = filter::Targets::default()
            .with_default(tracing::Level::ERROR)
            .with_target("syre", SYRE_LOG_LEVEL_FILE);
        let file_logger = tracing_appender::rolling::daily(config_dir, LOG_PREFIX);
        let (file_logger, _log_guard) = tracing_appender::non_blocking(file_logger);
        let file_logger = fmt::layer()
            .with_writer(file_logger)
            .with_timer(time::UtcTime::rfc_3339())
            .json()
            .with_filter(file_filter);

        #[cfg(debug_assertions)]
        let console_logger = fmt::layer()
            .with_writer(std::io::stdout)
            .with_timer(time::UtcTime::rfc_3339())
            .pretty()
            .with_filter(filter::EnvFilter::from_default_env());

        let subscriber = Registry::default().with(file_logger);

        #[cfg(debug_assertions)]
        let subscriber = subscriber.with(console_logger);

        tracing::subscriber::set_global_default(subscriber).unwrap();
        _log_guard
    }
}
