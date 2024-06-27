//! Setup functionality for the app.
use syre_local_database::state::ConfigState;
use tauri::Manager;

/// Runs setup tasks:
/// 1. Launches the update listener.
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some((_rx, _child)) = crate::db::start_database_if_needed(app.handle()) {
        tracing::debug!("initialized local database");
    } else {
        tracing::debug!("database already running");
    };

    let actor = crate::db::actor::Builder::new(app.handle().clone());
    std::thread::Builder::new()
        .name("syre desktop update listener".into())
        .spawn(move || actor.run())?;

    let main = app.get_webview_window("main").unwrap();
    main.listen(crate::db::FS_EVENT_TOPIC, move |event| {
        tracing::debug!(?event);
    });

    let db = app.state::<syre_local_database::Client>();
    let state = crate::State::new();
    if let ConfigState::Ok(local_config) = db.state().local_config().unwrap() {
        if let Some(user) = local_config.user {
            *state.user().lock().unwrap() = Some(user.clone());
            *state.projects().lock().unwrap() = db
                .user()
                .projects(user)
                .unwrap()
                .into_iter()
                .map(|(path, _)| path)
                .collect();
        }
    }

    assert!(app.manage(state));

    Ok(())
}
