//! Runs a local [`Database`].
//!
//! Must be run with the `server` feature enabled.

fn main() {
    #[cfg(feature = "server")]
    server::main();
    #[cfg(not(feature = "server"))]
    panic!("must be run with `server` feature enabled.");
}

#[cfg(feature = "server")]
mod server {
    use syre_local::{self as local, system::collections::ProjectManifest};
    use syre_project_daemon::server;

    /// Run the database with the default config.
    ///
    /// # Notes
    /// + Must run with the `server` feature enabled.
    pub fn main() {
        #[cfg(feature = "tracing")]
        logging::enable();

        let default_panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            panic_hook(panic_info);
            default_panic_hook(panic_info);
        }));

        let config = server::config::Builder::try_default().unwrap();
        let projects = match ProjectManifest::load_or_default() {
            Ok(projects) => projects.to_vec(),
            Err(err) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("could not load project manifest: {err:?}");
                vec![]
            }
        };

        let mut db = server::Builder::new(config.build());
        db.add_paths(projects);
        db.add_ignore_path(format!("**/{}*", local::constants::TEMPFILE_PREFIX))
            .unwrap();

        db.run().unwrap();
    }

    fn panic_hook(panic_info: &std::panic::PanicHookInfo) {
        let payload = if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
            Some(&**payload)
        } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
            Some(payload.as_str())
        } else {
            None
        };

        let location = panic_info.location().map(|location| location.to_string());
        #[cfg(feature = "tracing")]
        tracing::error!("local/project_daemon panicked at {location:?}: {payload:?}");
        // TODO: If `tracing` is not enabled?
    }

    #[cfg(feature = "tracing")]
    mod logging {
        use std::io;
        use syre_local::system::common;
        use tracing_subscriber::{
            EnvFilter, Registry,
            fmt::{self, time::UtcTime},
            prelude::*,
        };

        const LOG_PREFIX: &str = "project_daemon.local.log";

        /// Enable logging.
        pub fn enable() {
            let config_dir = common::config_dir_path().expect("could not get config dir path");
            let file_logger = tracing_appender::rolling::daily(config_dir, LOG_PREFIX);
            let (file_logger, _log_guard) = tracing_appender::non_blocking(file_logger);
            let file_logger = fmt::layer()
                .with_writer(file_logger)
                .with_timer(UtcTime::rfc_3339())
                .json();

            let console_logger = fmt::layer()
                .with_writer(io::stdout)
                .with_timer(UtcTime::rfc_3339())
                .pretty();

            let subscriber = Registry::default()
                .with(EnvFilter::from_default_env())
                .with(console_logger)
                .with(file_logger);

            tracing::subscriber::set_global_default(subscriber).unwrap();
        }
    }
}
