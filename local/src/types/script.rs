//! Script
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::Path;
use syre_core::error::Resource as ResourceError;
use syre_core::project::{ExcelTemplate, Script};
use syre_core::types::resource_map::values_only;
use syre_core::types::{ResourceId, ResourceMap};

#[derive(Serialize, Deserialize, Debug)]
pub enum ScriptKind<'a> {
    Script(Cow<'a, Script>),
    ExcelTemplate(Cow<'a, ExcelTemplate>),
}

impl<'a> From<Script> for ScriptKind<'a> {
    fn from(value: Script) -> Self {
        Self::Script(Cow::Owned(value))
    }
}

impl<'a> From<&'a Script> for ScriptKind<'a> {
    fn from(value: &'a Script) -> Self {
        Self::Script(Cow::Borrowed(value))
    }
}

impl<'a> From<ExcelTemplate> for ScriptKind<'a> {
    fn from(value: ExcelTemplate) -> Self {
        Self::ExcelTemplate(Cow::Owned(value))
    }
}
impl<'a> From<&'a ExcelTemplate> for ScriptKind<'a> {
    fn from(value: &'a ExcelTemplate) -> Self {
        Self::ExcelTemplate(Cow::Borrowed(value))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct ScriptStore {
    #[serde(with = "values_only")]
    scripts: ResourceMap<Script>,

    #[serde(with = "values_only")]
    excel_templates: ResourceMap<ExcelTemplate>,
}

impl ScriptStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns keys for all elements.
    pub fn keys(&self) -> Vec<&ResourceId> {
        self.scripts
            .keys()
            .chain(self.excel_templates.keys())
            .collect()
    }

    pub fn get(&self, key: &ResourceId) -> Option<ScriptKind> {
        if let Some(script) = self.scripts.get(key) {
            return Some(ScriptKind::Script(Cow::Borrowed(script)));
        }

        if let Some(template) = self.excel_templates.get(key) {
            return Some(ScriptKind::ExcelTemplate(Cow::Borrowed(template)));
        }

        None
    }

    pub fn remove(&mut self, key: &ResourceId) -> Option<ScriptKind> {
        if let Some(script) = self.scripts.remove(key) {
            return Some(ScriptKind::Script(Cow::Owned(script)));
        }

        if let Some(template) = self.excel_templates.remove(key) {
            return Some(ScriptKind::ExcelTemplate(Cow::Owned(template)));
        }

        None
    }

    pub fn scripts(&self) -> Vec<&Script> {
        self.scripts.values().collect()
    }

    pub fn excel_templates(&self) -> Vec<&ExcelTemplate> {
        self.excel_templates.values().collect()
    }

    pub fn insert_script(&mut self, script: Script) -> Option<Script> {
        self.scripts.insert(script.rid.clone(), script)
    }

    /// Inserts a script only if its path isn't yet in the collection.
    ///
    /// # Errors
    /// + [`ResourceError::AlreadyExists`] if a script with the same path is
    /// already present.
    pub fn insert_script_unique_path(&mut self, script: Script) -> Result<(), ResourceError> {
        if self.scripts_contain_path(&script.path) {
            return Err(ResourceError::already_exists(
                "`Script` with same path is already present",
            ));
        }

        self.scripts.insert(script.rid.clone(), script);

        Ok(())
    }

    pub fn remove_script(&mut self, script: &ResourceId) -> Option<Script> {
        self.scripts.remove(script)
    }

    pub fn scripts_contains_key(&self, script: &ResourceId) -> bool {
        self.scripts.contains_key(script)
    }

    /// Returns whether a script with the given path is registered.
    pub fn scripts_contain_path(&self, path: impl AsRef<Path>) -> bool {
        self.script_by_path(path).is_some()
    }

    pub fn get_script(&self, script: &ResourceId) -> Option<&Script> {
        self.scripts.get(script)
    }

    pub fn get_script_mut(&mut self, script: &ResourceId) -> Option<&mut Script> {
        self.scripts.get_mut(script)
    }

    /// Gets a script by its path if it is registered.
    pub fn script_by_path(&self, path: impl AsRef<Path>) -> Option<&Script> {
        let path = path.as_ref();
        for script in self.scripts.values() {
            if script.path == path {
                return Some(script);
            }
        }

        None
    }

    pub fn insert_excel_template(&mut self, template: ExcelTemplate) -> Option<ExcelTemplate> {
        self.excel_templates.insert(template.rid.clone(), template)
    }

    pub fn remove_excel_template(&mut self, template: &ResourceId) -> Option<ExcelTemplate> {
        self.excel_templates.remove(template)
    }

    pub fn get_excel_template(&self, rid: &ResourceId) -> Option<&ExcelTemplate> {
        for template in self.excel_templates.values() {
            if &template.rid == rid {
                return Some(template);
            }
        }

        None
    }
}
