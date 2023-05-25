//! User related commands
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommand {
    /// Get the active user
    GetActive,
}
