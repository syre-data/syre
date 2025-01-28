//! Interaction with a [`project watcher`](syre_project_watcher::server::Watcher).
pub mod actor;
mod init;

pub use init::start_project_watcher_if_needed;

/// Event to listen to to recieve file system event updates.
pub const FS_EVENT_TOPIC: &str = "fs-updates";
