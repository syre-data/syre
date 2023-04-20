//! Runs a [`Database`].
use std::io;
use thot_local_database::server::Database;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::fmt::Subscriber;

fn main() {
    // logging setup
    let logger = Subscriber::builder()
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(io::stdout) // write events to the console
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(logger).expect("could not create logger");

    // run database
    let mut db = Database::new();
    db.listen_for_commands();
}
