//! Local `Project` settings.
use serde::{Deserialize, Serialize};
use thot_core::types::{ResourceMap, UserPermissions};

#[derive(Serialize, Deserialize, PartialEq, Clone, Default, Debug)]
pub struct ProjectSettings {
    pub permissions: ResourceMap<UserPermissions>,
}

#[cfg(test)]
#[path = "./project_settings_test.rs"]
mod project_settings_test;
