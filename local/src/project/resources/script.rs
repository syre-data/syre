//! Local [`Script`].
use crate::common::scripts_file;
use crate::file_resource::LocalResource;
use crate::system::settings::user_settings::UserSettings;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use syre_core::error::{Error as CoreError, Resource as ResourceError};
use syre_core::project::Script as CoreScript;
use syre_core::types::resource_map::values_only;
use syre_core::types::ResourceMap;

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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(transparent)]
pub struct Scripts {
    #[serde(skip)]
    base_path: PathBuf,

    #[serde(with = "values_only")]
    scripts: ResourceMap<CoreScript>,
}

impl Scripts {
    pub fn new(path: PathBuf) -> Self {
        Self {
            base_path: path,
            scripts: ResourceMap::new(),
        }
    }

    pub fn load_from(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();
        let path = base_path.join(Self::rel_path());
        let fh = fs::OpenOptions::new().read(true).open(path)?;
        let Self {
            base_path: _,
            scripts,
        } = serde_json::from_reader(fh)?;

        Ok(Self { base_path, scripts })
    }

    pub fn save(&self) -> Result {
        fs::write(self.path(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }

    /// Returns whether a script with the given path is registered.
    pub fn contains_path(&self, path: impl AsRef<Path>) -> bool {
        self.by_path(path).is_some()
    }

    /// Gets a script by its path if it is registered.
    pub fn by_path(&self, path: impl AsRef<Path>) -> Option<&CoreScript> {
        let path = path.as_ref();
        for script in self.scripts.values() {
            if script.path == path {
                return Some(script);
            }
        }

        None
    }

    /// Inserts a script.
    ///
    /// # Errors
    /// + [`ResourceError::AlreadyExists`] if a script with the same path is
    /// already present.
    pub fn insert_script(&mut self, script: CoreScript) -> Result {
        if self.contains_path(&script.path) {
            return Err(CoreError::Resource(ResourceError::already_exists(
                "`Script` with same path is already present",
            ))
            .into());
        }

        self.scripts.insert(script.rid.clone(), script);
        Ok(())
    }
}

impl Deref for Scripts {
    type Target = ResourceMap<CoreScript>;

    fn deref(&self) -> &Self::Target {
        &self.scripts
    }
}

impl DerefMut for Scripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scripts
    }
}

impl LocalResource<ResourceMap<CoreScript>> for Scripts {
    fn rel_path() -> PathBuf {
        scripts_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}
