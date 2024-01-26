use crate::error::ScriptError;
use crate::types::{ResourceId, ResourceMap};
use chrono::prelude::*;
use has_id::HasId;
use serde_json::Value as JsValue;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use has_id::HasIdSerde;

#[cfg(feature = "serde")]
use crate::types::resource_map::values_only;

// **************
// *** Script ***
// **************

/// Represents a Script belonging to a specific project.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, HasIdSerde))]
#[derive(HasId, PartialEq, Debug, Clone)]
pub struct Script {
    #[id]
    pub rid: ResourceId,
    pub path: PathBuf,
    pub name: Option<String>,
    pub description: Option<String>,
    pub env: ScriptEnv,
    pub creator: Option<ResourceId>,
    created: DateTime<Utc>,
}

impl Script {
    pub fn new(path: impl Into<PathBuf>) -> StdResult<Script, ScriptError> {
        let path = path.into();
        let Some(file_name) = path.file_name() else {
            return Err(ScriptError::UnknownLanguage(None));
        };

        let env = ScriptEnv::new(Path::new(file_name))?;
        Ok(Script {
            rid: ResourceId::new(),
            path,
            name: None,
            description: None,
            creator: None,
            created: Utc::now(),
            env,
        })
    }

    /// Returns the date-time the script was created.
    /// This does not refer to the creation date-time of the script file,
    /// but rather the abstract Script object.
    pub fn created(&self) -> &DateTime<Utc> {
        &self.created
    }
}

// ***************
// *** Scripts ***
// ***************

/// Project scripts.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Scripts(#[cfg_attr(feature = "serde", serde(with = "values_only"))] ResourceMap<Script>);
impl Scripts {
    /// Returns whether a script with the given path is registered.
    pub fn contains_path(&self, path: impl AsRef<Path>) -> bool {
        self.by_path(path).is_some()
    }

    /// Gets a script by its path if it is registered.
    pub fn by_path(&self, path: impl AsRef<Path>) -> Option<&Script> {
        let path = path.as_ref();
        for script in self.values() {
            if &script.path == path {
                return Some(&script);
            }
        }

        None
    }
}

impl Deref for Scripts {
    type Target = ResourceMap<Script>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ResourceMap<Script>> for Scripts {
    fn from(scripts: ResourceMap<Script>) -> Self {
        Self(scripts)
    }
}

// ******************
// *** Script Env ***
// ******************

/// Defines the environment the script should run in.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Debug, Clone)]
pub struct ScriptEnv {
    /// Language of the script.
    pub language: ScriptLang,

    /// Command to run the script.
    pub cmd: String,

    /// Arguments passed to the command (`cmd`).
    pub args: Vec<String>,

    /// Environment variables.
    pub env: HashMap<String, String>,
}

impl ScriptEnv {
    /// Creates a new script environment for the given script.
    pub fn new(script: &Path) -> StdResult<Self, ScriptError> {
        let path_ext = script.extension();
        if path_ext.is_none() {
            return Err(ScriptError::UnknownLanguage(None));
        }

        // lang
        let path_ext = path_ext.unwrap();
        let language = ScriptLang::from_extension(path_ext);
        if language.is_none() {
            return Err(ScriptError::UnknownLanguage(Some(
                path_ext.to_str().unwrap().to_string(),
            )));
        }
        let language = language.unwrap();

        // cmd
        let cmd = match &language {
            ScriptLang::Python => "python3",
            ScriptLang::R => "Rscript",
        }
        .to_string();

        // args
        let args = Vec::new();

        // env
        let env = HashMap::new();

        Ok(ScriptEnv {
            language,
            cmd,
            args,
            env,
        })
    }
}

// *******************
// *** Script Lang ***
// *******************

/// Defines the language of the script.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptLang {
    Python,
    R,
}

impl ScriptLang {
    /// Returns the language type from a file extension
    /// or `None` if none match.
    #[tracing::instrument]
    pub fn from_extension(ext: &OsStr) -> Option<Self> {
        let ext = ext.to_ascii_lowercase();
        let Some(ext) = ext.as_os_str().to_str() else {
            return None;
        };

        match ext {
            "py" => Some(Self::Python),
            "r" => Some(Self::R),
            _ => None,
        }
    }

    /// Returns a list of supported extensions.
    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["py", "r"]
    }
}

// *************************
// *** Script Parameters ***
// *************************

pub type ScriptParameters = HashMap<String, JsValue>;
