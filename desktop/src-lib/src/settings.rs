use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
use syre_local as local;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub desktop: Result<Desktop, local::error::IoSerde>,
    pub runner: Result<Runner, local::error::IoSerde>,
}

impl User {
    pub fn replace_not_found_with_default(&mut self) {
        if let Err(err) = &self.desktop {
            if matches!(err, local::error::IoSerde::Io(io::ErrorKind::NotFound)) {
                self.desktop = Ok(Desktop::default());
            }
        }

        if let Err(err) = &self.runner {
            if matches!(err, local::error::IoSerde::Io(io::ErrorKind::NotFound)) {
                self.runner = Ok(Runner::default());
            }
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            desktop: Ok(Default::default()),
            runner: Ok(Default::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Desktop {
    /// Form input debounce in milliseconds.
    pub input_debounce_ms: usize,
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            input_debounce_ms: 250,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Runner {
    pub python_path: Option<PathBuf>,
    pub r_path: Option<PathBuf>,
}

impl From<local::system::config::runner_settings::Settings> for Runner {
    fn from(value: local::system::config::runner_settings::Settings) -> Self {
        Self {
            python_path: value.python_path,
            r_path: value.r_path,
        }
    }
}

impl Into<local::system::config::runner_settings::Settings> for Runner {
    fn into(self) -> local::system::config::runner_settings::Settings {
        local::system::config::runner_settings::Settings {
            python_path: self.python_path,
            r_path: self.r_path,
        }
    }
}
