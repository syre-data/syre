//! System settings for Thot.
pub mod runner_settings;
pub mod user_settings;

// Re-exports
pub use runner_settings::RunnerSettings;
pub use user_settings::UserSettings;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
