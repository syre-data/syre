//! Database that watches file system for changes, publishing them for clients.
pub(self) mod daemon;
mod state;
mod types;

pub use daemon::{Builder, Config, Daemon, config};
use state::State;
