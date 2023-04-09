//! Project template.
use crate::system::common::config_dir_path;
use crate::Result;
use cluFlock::FlockLock;
use has_id::{HasId, HasIdSerde};
use serde::{Deserialize, Serialize};
use settings_manager::error::Result as SettingsResult;
use settings_manager::{Priority as SettingsPriority, Settings, UserSettings};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use thot_core::graph::ResourceTree;
use thot_core::project::Project as CoreProject;
use thot_core::system::template::{Project as ProjectTemplate, ResourceTree as TreeTemplate};
use thot_core::types::ResourceId;

pub struct Project {
    file_lock: FlockLock<File>,
    rel_path: PathBuf,
    project: ProjectTemplate,
}

impl Project {
    /// Creates a new [`Project`](crate::project::Project) from the template.
    pub fn create_project<T>(&self, path: PathBuf) -> Result<(CoreProject, ResourceTree<T>)>
    where
        T: HasId<Id = ResourceId> + HasIdSerde<'static, Id = ResourceId>,
    {
        todo!();
        // let mut project = CoreProject::new(&self.name);
        // project.description = self.project.description.clone();
        // project.data_root = self.project.data_root.clone();
        // project.universal_root = self.universal_root.clone();
        // project.analysis_root = self.project.analysis_root.clone();

        // let graph = ResourceTree::to_tree(graph);

        // Ok((project, graph))
    }
}

impl Deref for Project {
    type Target = ProjectTemplate;

    fn deref(&self) -> &Self::Target {
        &self.project
    }
}

impl DerefMut for Project {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.project
    }
}

impl Settings<ProjectTemplate> for Project {
    fn settings(&self) -> &ProjectTemplate {
        &self.project
    }

    fn file(&self) -> &File {
        &*self.file_lock
    }

    fn file_mut(&mut self) -> &mut File {
        &mut *self.file_lock
    }

    fn file_lock(&self) -> &FlockLock<File> {
        &self.file_lock
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::User
    }
}

impl UserSettings<ProjectTemplate> for Project {
    /// Returns the base path to the settings file.
    fn base_path() -> &'static Path {
        let mut path = config_dir_path().expect("could not get config path");
        path.push("templates");

        &path
    }

    /// Returns the relative path for the settings.
    fn rel_path(&self) -> &Path {
        &self.rel_path
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
