#![feature(assert_matches)]
//! File system event handler.
mod actor;
pub mod event;
pub(crate) mod watcher;

#[cfg(target_os = "windows")]
mod preprocess_file_system_events_windows;

pub use watcher::{config, Builder, FsWatcher};
