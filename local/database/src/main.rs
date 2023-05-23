//! Runs a local [`Database`].
use std::io;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{Registry, Layer};
use thot_local_database::server::Database;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::fmt;
use thot_local::system::common;

const LOG_PREFIX: &str = "database.local.log";
const MAX_LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

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

    let subscriber = Registry::default()
        .with(console_logger)
        .with(file_logger);

    tracing::subscriber::set_global_default(subscriber).expect("could not create logger");

    // run database
    let mut db = Database::new();
    db.listen_for_commands();
}
