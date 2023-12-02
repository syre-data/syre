//! Collection of templates.
use crate::file_resource::SystemResource;
use crate::system::common::config_dir_path;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use thot_core::system::template::Project as ProjectTemplate;
use thot_core::types::ResourceId;

pub type TemplateMap = HashMap<ResourceId, ProjectTemplate>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Templates(TemplateMap);
impl Templates {
    pub fn load() -> Result<Self> {
        let file = fs::File::open(Self::path())?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save(&self) -> Result {
        fs::write(Self::path(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl SystemResource<TemplateMap> for Templates {
    /// Returns the path to the system settings file.
    fn path() -> PathBuf {
        let settings_dir = config_dir_path().expect("could not get settings directory");
        settings_dir.join("templates.json")
    }
}

impl Deref for Templates {
    type Target = TemplateMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Templates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
