//! Collection of templates.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use settings_manager::settings::Settings;
use settings_manager::system_settings::SystemSettings;
use settings_manager::types::Priority as SettingsPriority;
use std::collections::HashMap;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::system::template::Project as ProjectTemplate;
use thot_core::types::ResourceId;

pub type TemplateMap = HashMap<ResourceId, ProjectTemplate>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Templates {
    file_lock: FlockLock<File>,
    pub templates: TemplateMap,
}

impl Settings<TemplateMap> for Templates {
    fn settings(&self) -> &TemplateMap {
        &self.templates
    }

    fn file(&self) -> &File {
        &self.file_lock
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

impl SystemSettings<TemplateMap> for Templates {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("templates.json")
    }
}

impl Deref for Templates {
    type Target = TemplateMap;

    fn deref(&self) -> &Self::Target {
        &self.templates
    }
}

impl DerefMut for Templates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.templates
    }
}

#[cfg(test)]
#[path = "./templates_test.rs"]
mod templates_test;
