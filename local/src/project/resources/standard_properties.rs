use crate::system::settings::user_settings::UserSettings;
use crate::Result;
use settings_manager::system_settings::Loader;
use thot_core::project::StandardProperties as CoreStandardProperties;
use thot_core::types::{Creator, UserId};

pub struct StandardProperties;

impl StandardProperties {
    /// Creates a new [`StandardProperties`] with fields actively filled from system settings.
    pub fn new() -> Result<CoreStandardProperties> {
        let settings: UserSettings = Loader::load_or_create::<UserSettings>()?.into();
        let creator = match settings.active_user.as_ref() {
            Some(uid) => Some(UserId::Id(uid.clone().into())),
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
