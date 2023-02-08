//! Generic database commands.
use serde::{Deserialize, Serialize};

/// Generic database commands.
#[derive(Serialize, Deserialize)]
pub enum DatabaseCommand {
    /// Used to kill the `Database`.
    Kill,

    /// Used to identify the running `Database`.
    Id,
}

#[cfg(test)]
#[path = "./database_test.rs"]
mod database_test;
