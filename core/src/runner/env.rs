//! Environment variables for runner.
pub static CONTAINER_ID_KEY: &str = "THOT_CONTAINER_ID";

#[cfg(test)]
#[path = "./env_test.rs"]
mod env_test;
