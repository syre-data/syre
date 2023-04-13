//! Common functionality.
use settings_manager::Result;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use thot_local::system::common;

/// Path to user config directory.
pub fn users_config_dir() -> Result<PathBuf> {
    let mut path = common::config_dir_path()?;
    path.push("user_config");
    Ok(path)
}

/// Path to a user's config directory.
pub fn user_config_dir(user: &ResourceId) -> Result<PathBuf> {
    let mut path = users_config_dir()?;
    path.push(user.to_string());
    Ok(path)
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
