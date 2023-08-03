//! User settings.
use super::{GeneralSettings, HasUser};
use serde::{Deserialize, Serialize};
use thot_core::types::ResourceId;

/// A user's settings.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct UserSettings {
    user: ResourceId,
    pub general: GeneralSettings,
}

impl HasUser for UserSettings {
    fn new(user: ResourceId) -> Self {
        Self {
            user,
            general: GeneralSettings::new(),
        }
    }

    fn user(&self) -> &ResourceId {
        &self.user
    }
}
