fn main() {
    logging::enable();

    let (_command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    let db = syre_resource_db::Builder::new(command_rx);
    db.run().unwrap();
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
