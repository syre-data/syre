//! Common functionality related to the Syre runner.
use super::CONTAINER_ID_KEY;
use std::env;

/// Returns whether the script is being run in developement mode.
pub fn dev_mode() -> bool {
    // return true if a container id is not set
    env::var(CONTAINER_ID_KEY).is_err()
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
