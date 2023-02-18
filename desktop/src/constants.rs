//! Constat values.
pub static MESSAGE_TIMEOUT: u32 = 5_000;
pub static SCRIPT_DISPLAY_NAME_MAX_LENGTH: usize = 30;

#[cfg(test)]
#[path = "./constants_test.rs"]
mod constants_test;
