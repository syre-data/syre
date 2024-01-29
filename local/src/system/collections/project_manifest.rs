//! Projects collection.
use crate::error::IoSerde;
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufReader};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(transparent)]
pub struct ProjectManifest(Vec<PathBuf>);

impl ProjectManifest {
    pub fn load() -> Result<Self, IoSerde> {
        let file = fs::File::open(Self::path())?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn load_or_default() -> Result<Self, IoSerde> {
        match fs::File::open(Self::path()) {
            Ok(file) => {
                let reader = BufReader::new(file);
                Ok(serde_json::from_reader(reader)?)
            }

            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn save(&self) -> Result<(), IoSerde> {
        fs::write(Self::path(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.retain(|project| {
            let project: &Path = project.as_ref();
            project != path
        });
    }
}

impl Deref for ProjectManifest {
    type Target = Vec<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProjectManifest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SystemResource<Vec<PathBuf>> for ProjectManifest {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().unwrap();
        settings_dir.join("project_manifest.json")
    }
}
