use crate::system::settings::user_settings::UserSettings;
use crate::Result;
use settings_manager::SystemSettings;
use thot_core::project::StandardProperties as CoreStandardProperties;
use thot_core::types::{Creator, UserId};

pub struct StandardProperties;

impl StandardProperties {
    /// Creates a new [`StandardProperties`] with fields actively filled from system settings.
    pub fn new() -> Result<CoreStandardProperties> {
        let settings = UserSettings::load()?;
        let creator = match settings.active_user {
            Some(uid) => Some(UserId::Id(uid.into())),
            None => None,
        };

        let creator = Creator::User(creator);
        let mut props = CoreStandardProperties::new();
        props.creator = creator;

        Ok(props)
    }
}

#[cfg(test)]
#[path = "standard_properties_test.rs"]
mod standard_properties_test;
