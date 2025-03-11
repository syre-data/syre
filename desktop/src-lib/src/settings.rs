pub use app::Settings as App;
pub use project::Settings as Project;
pub use user::Settings as User;

pub mod user {
    use serde::{Deserialize, Serialize};
    use std::{io, num::NonZeroUsize, path::PathBuf};
    use syre_local as local;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Settings {
        pub desktop: Result<Desktop, local::error::IoSerde>,
        pub runner: Result<Runner, local::error::IoSerde>,
    }

    impl Settings {
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

    impl Default for Settings {
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
        pub continue_on_error: bool,
        pub max_tasks: Option<NonZeroUsize>,
        pub disable_analysis_after: local::system::config::runner_settings::DisableAnalysisAfter,
    }

    impl From<local::system::config::runner_settings::Settings> for Runner {
        fn from(value: local::system::config::runner_settings::Settings) -> Self {
            Self {
                python_path: value.python_path,
                r_path: value.r_path,
                continue_on_error: value.continue_on_error,
                max_tasks: value.max_tasks,
                disable_analysis_after: value.disable_analysis_after,
            }
        }
    }

    impl Into<local::system::config::runner_settings::Settings> for Runner {
        fn into(self) -> local::system::config::runner_settings::Settings {
            local::system::config::runner_settings::Settings {
                python_path: self.python_path,
                r_path: self.r_path,
                continue_on_error: self.continue_on_error,
                max_tasks: self.max_tasks,
                disable_analysis_after: self.disable_analysis_after,
            }
        }
    }
}

pub mod project {
    use serde::{Deserialize, Serialize};
    use std::{io, num::NonZeroUsize, path::PathBuf};
    use syre_local as local;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Settings {
        pub desktop: Result<Desktop, local::error::IoSerde>,
        pub runner: Result<Runner, local::error::IoSerde>,
    }

    impl Settings {
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

    impl Default for Settings {
        fn default() -> Self {
            Self {
                desktop: Ok(Default::default()),
                runner: Ok(Default::default()),
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Default, Debug)]
    pub struct Desktop {
        pub asset_drag_drop_kind: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Default, Debug)]
    pub struct Runner {
        pub python_path: Option<PathBuf>,
        pub r_path: Option<PathBuf>,
        pub continue_on_error: Option<bool>,
        pub max_tasks: Option<NonZeroUsize>,
    }

    impl From<local::project::config::runner_settings::Settings> for Runner {
        fn from(value: local::project::config::runner_settings::Settings) -> Self {
            Self {
                python_path: value.python_path,
                r_path: value.r_path,
                continue_on_error: value.continue_on_error,
                max_tasks: value.max_tasks,
            }
        }
    }

    impl Into<local::project::config::runner_settings::Settings> for Runner {
        fn into(self) -> local::project::config::runner_settings::Settings {
            local::project::config::runner_settings::Settings {
                python_path: self.python_path,
                r_path: self.r_path,
                continue_on_error: self.continue_on_error,
                max_tasks: self.max_tasks,
            }
        }
    }
}

pub mod app {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Settings {
        pub update_channel: UpdateChannel,
    }

    #[derive(Serialize, Deserialize, Clone, Copy, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum UpdateChannel {
        Stable,
        Debug,

        // Do not update app.
        None,
    }

    impl Default for UpdateChannel {
        fn default() -> Self {
            Self::Stable
        }
    }

    impl std::fmt::Display for UpdateChannel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let out = match self {
                UpdateChannel::Stable => "stable",
                UpdateChannel::Debug => "debug",
                UpdateChannel::None => panic!("{self:?} can not be formatted"),
            };

            write!(f, "{out}")
        }
    }
}
