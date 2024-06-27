use std::path::PathBuf;

pub trait UserSettingsFile {
    /// Returns the path to the settings file relative to the user's config directory.
    /// The file should reside at <config_dir>/<users_dir>/<settings_file>.
    fn settings_file() -> PathBuf;
}
