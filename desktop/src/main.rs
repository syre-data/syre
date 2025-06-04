use leptos::prelude::*;

fn main() {
    #[cfg(debug_assertions)]
    tracing::enable();
    console_error_panic_hook::set_once();
    mount_to_body(syre_desktop_ui::App);
}

#[cfg(debug_assertions)]
mod tracing {
    use tracing_subscriber::{filter, fmt::time::UtcTime, prelude::*};

    pub fn enable() {
        let target_filter = filter::Targets::new()
            .with_target("syre", tracing::Level::TRACE)
            .with_target("leptos", tracing::Level::TRACE);
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false) // Only partially supported across browsers
            .with_timer(UtcTime::rfc_3339())
            .pretty()
            .with_writer(tracing_web::MakeWebConsoleWriter::new()); // write events to the console

        let perf_layer = tracing_web::performance_layer()
            .with_details_from_fields(tracing_subscriber::fmt::format::Pretty::default());

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(perf_layer)
            .with(target_filter)
            .init();
    }
}
