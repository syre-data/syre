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
use syre_core::project::Project as CoreProject;

/// Represents a Syre project.
pub struct Project {
    inner: CoreProject,
    base_path: PathBuf,
    settings: ProjectSettings,
}

impl Project {
    /// Create a new `Project` with the given path.
    /// Name of the `Project` is taken from the last component of the path.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
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
            inner: CoreProject::new(name),
            settings: ProjectSettings::new(),
        })
    }

    /// Create a new local project from the provided project.
    pub fn from(path: impl Into<PathBuf>, project: CoreProject) -> Self {
        Self {
            base_path: path.into(),
            inner: project,
            settings: ProjectSettings::new(),
        }
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
            inner: project,
            settings,
        })
    }

    /// Save all data.
    pub fn save(&self) -> StdResult<(), IoSerdeError> {
        let project_path = <Project as LocalResource<CoreProject>>::path(self);
        let settings_path = <Project as LocalResource<ProjectSettings>>::path(self);
        let Some(parent) = project_path.parent() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "project path does not have a parent",
            )
            .into());
        };

        fs::create_dir_all(parent)?;
        fs::write(project_path, serde_json::to_string_pretty(&self.inner)?)?;
        fs::write(settings_path, serde_json::to_string_pretty(&self.settings)?)?;
        Ok(())
    }

    pub fn properties(&self) -> &CoreProject {
        &self.inner
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

    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
        self.base_path = path.into();
    }

    /// Get the full path of the data root.
    pub fn data_root_path(&self) -> PathBuf {
        self.base_path.join(&self.data_root)
    }

    /// Get the full path of the analysis root.
    pub fn analysis_root_path(&self) -> Option<PathBuf> {
        let Some(analysis_root) = self.analysis_root.as_ref() else {
            return None;
        };

        Some(self.base_path.join(analysis_root))
    }

    /// Breaks self into parts.
    ///
    /// # Returns
    /// Tuple of (properties, settings, base path).
    pub fn into_parts(self) -> (CoreProject, ProjectSettings, PathBuf) {
        let Self {
            inner,
            base_path,
            settings,
        } = self;

        (inner, settings, base_path)
    }
}

impl Project {
    /// Only load the project's properties.
    pub fn load_from_properties_only(
        base_path: impl Into<PathBuf>,
    ) -> StdResult<CoreProject, IoSerdeError> {
        let base_path = fs::canonicalize(base_path.into())?;
        let project_path = base_path.join(<Project as LocalResource<CoreProject>>::rel_path());
        let project_file = fs::File::open(project_path)?;
        let project_reader = BufReader::new(project_file);
        let project = serde_json::from_reader(project_reader)?;
        Ok(project)
    }
}

impl Deref for Project {
    type Target = CoreProject;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Project {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Into<CoreProject> for Project {
    fn into(self: Self) -> CoreProject {
        self.inner
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
