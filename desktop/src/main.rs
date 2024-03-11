#![feature(mutex_unlock)]
//! User interface for Syre Desktop.
mod actions;
mod app;
mod commands;
mod common;
mod components;
mod constants;
mod error;
mod hooks;
mod lib;
mod navigation;
mod pages;
mod routes;
mod widgets;

pub use error::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeConsoleWriter};

const MAX_LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

fn main() {
    // logging setup
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter) // write events to the console
        .with_filter(MAX_LOG_LEVEL);

    let perf_layer = performance_layer()
        .with_details_from_fields(Pretty::default())
        .with_filter(MAX_LOG_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    yew::Renderer::<app::App>::new().render();
}
