use leptos::*;

fn main() {
    #[cfg(debug_assertions)]
    tracing::enable();
    console_error_panic_hook::set_once();
    mount_to_body(syre_desktop_ui::App);
}

#[cfg(debug_assertions)]
mod tracing {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::fmt::time::UtcTime;
    use tracing_subscriber::prelude::*;
    use tracing_web::MakeConsoleWriter;

    const MAX_LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

    pub fn enable() {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false) // Only partially supported across browsers
            .with_timer(UtcTime::rfc_3339())
            .pretty()
            .with_writer(MakeConsoleWriter) // write events to the console
            .with_filter(MAX_LOG_LEVEL);

        tracing_subscriber::registry().with(fmt_layer).init();
    }
}
