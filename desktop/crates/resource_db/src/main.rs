use std::{
    io::{self, Write},
    path::PathBuf,
    thread,
};
use syre_desktop_resource_db as db;

/// Launch a resource database and accept user input to query it.
///
/// # Notes
/// + Must run with the `server` and `client` features enabled.
fn main() {
    logging::enable();

    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

    let db = db::Builder::new(command_rx);
    thread::Builder::new()
        .name("syre desktop resource database".to_string())
        .spawn(move || db.run().unwrap())
        .unwrap();

    let client = db::Client::new(command_tx);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        let mut query = String::new();
        stdin.read_line(&mut query).unwrap();
        if query.trim().is_empty() {
            continue;
        }

        if let Some(search) = query.strip_prefix("search(") {
            let Some((project, search)) = search.split_once("):") else {
                io::stderr().write_all("invalid query".as_bytes()).unwrap();
                continue;
            };

            let response = client
                .search_project(search.trim().to_string(), PathBuf::from(project))
                .unwrap();
            let out = format!("{response:?}\n\n");
            stdout.write_all(out.as_bytes()).unwrap();
        } else if let Some(search) = query.strip_prefix("search:") {
            let response = client.search(search.trim().to_string()).unwrap();
            let out = format!("{response:?}\n\n");
            stdout.write_all(out.as_bytes()).unwrap();
        } else {
            let response = client.query(query).unwrap();
            let out = format!("{response:?}\n\n");
            stdout.write_all(out.as_bytes()).unwrap();
        }
    }
}

mod logging {
    use std::io;
    use tracing_subscriber::{
        fmt::{self, time::UtcTime},
        prelude::*,
        EnvFilter, Layer, Registry,
    };

    /// Enable logging.
    pub fn enable() {
        let console_logger = fmt::layer()
            .with_writer(io::stdout)
            .with_timer(UtcTime::rfc_3339())
            .pretty()
            .with_filter(EnvFilter::from_default_env());

        let subscriber = Registry::default().with(console_logger);
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}
