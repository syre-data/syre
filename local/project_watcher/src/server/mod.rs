//! Database that watches file system for changes, publishing them for clients.
mod state;
mod types;
pub(self) mod watcher;

use state::State;
pub use watcher::{config, Builder, Config, Watcher};
