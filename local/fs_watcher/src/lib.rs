#![feature(assert_matches)]
//! File system event handler.
mod actor;
pub(crate) mod command;
pub mod error;
pub(crate) mod event;
pub(crate) mod watcher;

#[cfg(target_os = "windows")]
mod preprocess_file_system_events_windows;

pub use command::Command;
pub use error::{Error, Result};
pub use event::app::Event;
pub use watcher::FsWatcher;
