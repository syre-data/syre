//! Environment variables for runner.
pub struct ThotEnv;

impl ThotEnv {
    pub fn original_dir_key() -> String {
        String::from("THOT_ORIGINAL_DIR")
    }

    pub fn container_id_key() -> String {
        String::from("THOT_CONTAINER_ID")
    }
}

#[cfg(test)]
#[path = "./env_test.rs"]
mod env_test;
