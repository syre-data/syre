//! Common functionality related to the Thot runner.
use super::env::ThotEnv;
use std::env;

/// Returns whether the script is being run in developement mode.
pub fn dev_mode() -> bool {
    // return true if a container id is not set
    env::var(ThotEnv::container_id_key()).is_err()
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
