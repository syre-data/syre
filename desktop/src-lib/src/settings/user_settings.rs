//! User settings.
use super::GeneralSettings;
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

/// A user's settings.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct UserSettings {
    pub user: ResourceId,
    pub general: GeneralSettings,
}

impl UserSettings {
    pub fn new(user: ResourceId) -> Self {
        Self {
            user,
            general: GeneralSettings::new(),
        }
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self::new(ResourceId::new())
    }
}

#[cfg(test)]
#[path = "./user_settings_test.rs"]
mod user_settings_test;
