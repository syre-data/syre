//! Interaction with a [`project daemon`](syre_project_daemon::server::Daemon).
pub mod actor;
mod init;

pub use init::start_project_daemon_if_needed;

/// Event to listen to to recieve file system event updates.
pub const FS_EVENT_TOPIC: &str = "fs-updates";
