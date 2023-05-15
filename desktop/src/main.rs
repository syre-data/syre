#![feature(mutex_unlock)]
//! User interface for Thot Desktop.
mod app;
mod commands;
mod common;
mod components;
mod constants;
mod error;
mod hooks;
mod navigation;
mod pages;
mod routes;
mod widgets;

use app::App;
pub use error::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeConsoleWriter};

fn main() {
    // logging setup
    let max_log_level = LevelFilter::DEBUG;
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter) // write events to the console
        .with_filter(max_log_level);

    let perf_layer = performance_layer()
        .with_details_from_fields(Pretty::default())
        .with_filter(max_log_level);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    yew::Renderer::<App>::new().render();
}
