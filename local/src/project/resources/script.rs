//! Local [`Script`].
use crate::common::scripts_file;
use crate::file_resource::LocalResource;
use crate::system::settings::user_settings::UserSettings;
use crate::types::script::ScriptStore;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::{ExcelTemplate, Script as CoreScript};
use syre_core::types::resource_map::values_only;
use syre_core::types::{ResourceId, ResourceMap};

// **************
// *** Script ***
// **************

pub struct Script;
impl Script {
    /// Creates a new [`Script`] with the `creator` field matching the current active creator.
    pub fn new(path: impl Into<PathBuf>) -> Result<CoreScript> {
        let settings = UserSettings::load()?;
        let creator = settings.active_user.clone().map(|c| c.into());

        let mut script = match CoreScript::from_path(path) {
            Ok(script) => script,
            Err(err) => return Err(CoreError::Script(err).into()),
        };

        script.creator = creator;
        Ok(script)
    }
}

// ***************
// *** Scripts ***
// ***************

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(transparent)]
pub struct Scripts {
    #[serde(skip)]
    base_path: PathBuf,
    inner: ScriptStore,
}

impl Scripts {
    pub fn new(path: PathBuf) -> Self {
        Self {
            base_path: path,
            inner: ScriptStore::new(),
        }
    }

    pub fn load_from(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();
        let path = base_path.join(Self::rel_path());
        let fh = fs::OpenOptions::new().read(true).open(path)?;
        let inner = serde_json::from_reader(fh)?;

        Ok(Self { base_path, inner })
    }

    pub fn save(&self) -> Result {
        fs::write(self.path(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl Deref for Scripts {
    type Target = ScriptStore;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Scripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl LocalResource<ScriptStore> for Scripts {
    fn rel_path() -> PathBuf {
        scripts_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}
