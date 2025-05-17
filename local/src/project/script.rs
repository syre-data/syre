//! Local [`Script`].
#[cfg(feature = "fs")]
use crate::system::config::Config;
#[cfg(feature = "fs")]
use std::path::PathBuf;
#[cfg(feature = "fs")]
use syre_core::{error::Error as CoreError, project::Script as CoreScript};

pub struct Script;

#[cfg(feature = "fs")]
impl Script {
    /// Creates a new [`Script`] with the `creator` field matching the current active creator.
    pub fn new(path: impl Into<PathBuf>) -> crate::Result<CoreScript> {
        let config = Config::load()?;
        let creator = config.user.clone().map(|c| c.into());

        let mut script = match CoreScript::from_path(path) {
            Ok(script) => script,
            Err(err) => return Err(CoreError::Analysis(err).into()),
        };

        script.creator = creator;
        Ok(script)
    }
}
