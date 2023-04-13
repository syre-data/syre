use std::path::PathBuf;

pub trait UserSettingsFile {
    /// Returns the path to the settings file relative to the user's config directory.
    /// The file should reside at <config_dir>/<users_dir>/<settings_file>.
    fn settings_file() -> PathBuf;
}

#[cfg(test)]
#[path = "./user_settings_file_test.rs"]
mod user_settings_file_test;
