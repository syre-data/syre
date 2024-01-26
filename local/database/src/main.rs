//! Runs a local [`Database`].
use std::io;
use thot_local::system::common;
use thot_local_database::server::Database;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{Layer, Registry};

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
        .with_filter(MAX_LOG_LEVEL);

    let console_logger = fmt::layer()
        .with_writer(io::stdout)
        .with_timer(UtcTime::rfc_3339())
        .pretty()
        .with_filter(MAX_LOG_LEVEL);

    let subscriber = Registry::default().with(console_logger).with(file_logger);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        panic_hook(panic_info);
        default_panic_hook(panic_info);
    }));

    // run database
    let mut db = Database::new();
    db.start();
}

fn panic_hook(panic_info: &std::panic::PanicInfo) {
    let payload = if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
        Some(&**payload)
    } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
        Some(payload.as_str())
    } else {
        None
    };

    let location = panic_info.location().map(|location| location.to_string());
    tracing::error!("local/database panicked: {location:?} : {payload:?}");
}
