//! Project and project settings.
use crate::common::{project_file, project_settings_file};
use crate::error::IoSerde as IoSerdeError;
use crate::file_resource::LocalResource;
use crate::types::ProjectSettings;
use crate::Result;
use std::fs;
use std::io::{self, BufReader};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use thot_core::project::Project as CoreProject;

/// Represents a Thot project.
pub struct Project {
    base_path: PathBuf,
    project: CoreProject,
    settings: ProjectSettings,
}

impl Project {
    /// Create a new `Project` with the given path.
    /// Name of the `Project` is taken from the last component of the path.
    pub fn new(path: PathBuf) -> Result<Self> {
        let Some(name) = path.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "file name could not be extracted from path",
            )
            .into());
        };

        let Some(name) = name.to_str() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "file name could not be converted to string",
            )
            .into());
        };

        let name = name.to_string();
        Ok(Self {
            base_path: path,
            project: CoreProject::new(name),
            settings: ProjectSettings::default(),
        })
    }

    pub fn load_from(base_path: impl Into<PathBuf>) -> StdResult<Self, IoSerdeError> {
        let base_path = fs::canonicalize(base_path.into())?;
        let project_path = base_path.join(<Project as LocalResource<CoreProject>>::rel_path());
        let settings_path = base_path.join(<Project as LocalResource<ProjectSettings>>::rel_path());

        let project_file = fs::File::open(project_path)?;
        let settings_file = fs::File::open(settings_path)?;

        let project_reader = BufReader::new(project_file);
        let settings_reader = BufReader::new(settings_file);

        let project = serde_json::from_reader(project_reader)?;
        let settings = serde_json::from_reader(settings_reader)?;

        Ok(Self {
            base_path,
            project,
            settings,
        })
    }

    /// Save all data.
    pub fn save(&self) -> Result {
        let project_path = <Project as LocalResource<CoreProject>>::path(self);
        let settings_path = <Project as LocalResource<ProjectSettings>>::path(self);

        fs::create_dir_all(project_path.parent().expect("invalid path"))?;
        fs::write(project_path, serde_json::to_string_pretty(&self.project)?)?;
        fs::write(settings_path, serde_json::to_string_pretty(&self.settings)?)?;
        Ok(())
    }

    pub fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut ProjectSettings {
        &mut self.settings
    }

    pub fn base_path(&self) -> &Path {
        self.base_path.as_path()
    }

    /// Get the full path of the data root.
    pub fn data_root_path(&self) -> Option<PathBuf> {
        let Some(data_root) = self.data_root.as_ref() else {
            return None;
        };

        Some(self.base_path.join(data_root))
    }

    /// Get the full path of the analysis root.
    pub fn analysis_root_path(&self) -> Option<PathBuf> {
        let Some(analysis_root) = self.analysis_root.as_ref() else {
            return None;
        };

        Some(self.base_path.join(analysis_root))
    }
}

impl Deref for Project {
    type Target = CoreProject;

    fn deref(&self) -> &Self::Target {
        &self.project
    }
}

impl DerefMut for Project {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.project
    }
}

impl Into<CoreProject> for Project {
    fn into(self: Self) -> CoreProject {
        self.project
    }
}

impl LocalResource<CoreProject> for Project {
    fn rel_path() -> PathBuf {
        project_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl LocalResource<ProjectSettings> for Project {
    fn rel_path() -> PathBuf {
        project_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}
