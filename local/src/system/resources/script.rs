//! Local [`Script`].
use crate::system::settings::user_settings::UserSettings;
use crate::Result;
use settings_manager::system_settings::Loader;
use thot_core::project::Script as CoreScript;
use thot_core::types::ResourcePath;

// **************
// *** Script ***
// **************

pub struct Script;
impl Script {
    /// Creates a new [`Script`] with the `creator` field matching the current active creator.
    pub fn new(path: ResourcePath) -> Result<CoreScript> {
        let settings: UserSettings = Loader::load_or_create::<UserSettings>()?.into();
        let creator = settings.active_user.clone().map(|c| c.into());

        let mut script = Script::new(path)?;
        script.creator = creator;
        Ok(script)
    }
}

#[cfg(test)]
#[path = "./script_test.rs"]
mod script_test;
