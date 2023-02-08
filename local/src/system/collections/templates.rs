//! Collection of templates.
use crate::system::common::config_dir_path;
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use serde::{Deserialize, Serialize};
use settings_manager::settings::Settings;
use settings_manager::system_settings::{LockSettingsFile, SystemSettings};
use settings_manager::types::Priority as SettingsPriority;
use settings_manager::Result as SettingsResult;
use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::system::template::Template;
use thot_core::types::{resource_map::values_only, ResourceId};

pub type TemplateMap = HashMap<ResourceId, Template>;

#[derive(Serialize, Deserialize, Derivative, Default)]
#[derivative(Debug)]
pub struct Templates {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(with = "values_only")]
    pub templates: TemplateMap,
}

impl Settings for Templates {
    fn store_lock(&mut self, file_lock: FlockLock<File>) {
        self._file_lock = Some(file_lock);
    }

    fn controls_file(&self) -> bool {
        self._file_lock.is_some()
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::User
    }
}

impl SystemSettings for Templates {
    /// Returns the path to the system settings file.
    fn path() -> SettingsResult<PathBuf> {
        let settings_dir = config_dir_path()?;
        Ok(settings_dir.join("templates.json"))
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

impl LockSettingsFile for Templates {}

#[cfg(test)]
#[path = "./templates_test.rs"]
mod templates_test;
