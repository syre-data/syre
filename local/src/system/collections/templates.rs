//! Collection of templates.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use settings_manager::locked::system_settings::SystemSettings;
use settings_manager::LockedSettings;
use std::collections::HashMap;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::system::template::Project as ProjectTemplate;
use thot_core::types::ResourceId;

pub type TemplateMap = HashMap<ResourceId, ProjectTemplate>;

#[derive(Derivative, LockedSettings)]
#[derivative(Debug)]
pub struct Templates {
    #[locked_settings(file_lock = "TemplateMap")]
    file_lock: FlockLock<File>,

    #[locked_settings(priority = "User")]
    templates: TemplateMap,
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
