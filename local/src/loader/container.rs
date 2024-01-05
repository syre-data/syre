pub use error::{AssetFile as AssetFileError, Error, Properties as PropertiesError};
use serde::de::DeserializeOwned;

use crate::file_resource::LocalResource;
use crate::project::resources::container::{Container, StoredContainerProperties};
use crate::types::ContainerSettings;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use thot_core::project::container::{AssetMap, Container as CoreContainer};

/// Loads a [`Container`].
pub struct Loader;
impl Loader {
    pub fn load(base_path: impl AsRef<Path>) -> Result<Container, Error> {
        let base_path = base_path.as_ref();
        let base_path = match fs::canonicalize(base_path) {
            Ok(base_path) => base_path,
            Err(err) => return Err(Error::Root(err.kind())),
        };

        let properties_path =
            base_path.join(<Container as LocalResource<StoredContainerProperties>>::rel_path());

        let assets_path = base_path.join(<Container as LocalResource<AssetMap>>::rel_path());
        let settings_path =
            base_path.join(<Container as LocalResource<ContainerSettings>>::rel_path());

        let container = Self::load_json::<StoredContainerProperties>(properties_path);
        let assets = Self::load_json(assets_path);
        let settings = Self::load_json(settings_path);

        let properties = (container, assets, settings);
        let (Ok(container), Ok(assets), Ok(settings)) = properties else {
            let container = match properties.0 {
                Ok(_) => None,
                Err(err) => Some(err),
            };

            let assets = match properties.1 {
                Ok(_) => None,
                Err(err) => Some(err),
            };

            let settings = match properties.2 {
                Ok(_) => None,
                Err(err) => Some(err),
            };

            return Err(Error::Properties {
                container,
                assets,
                settings,
            });
        };

        let container = CoreContainer {
            rid: container.rid,
            properties: container.properties,
            assets,
            scripts: container.scripts,
        };

        Ok(Container {
            base_path,
            container,
            settings,
        })
    }

    fn load_json<T: DeserializeOwned>(path: PathBuf) -> Result<T, PropertiesError> {
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                return Err(PropertiesError::Io {
                    path,
                    kind: err.kind(),
                });
            }
        };

        let reader = BufReader::new(file);
        match serde_json::from_reader(reader) {
            Ok(obj) => Ok(obj),
            Err(err) => Err(PropertiesError::Serde {
                path,
                err: err.to_string(),
            }),
        }
    }
}

pub struct AssetValidator;
impl AssetValidator {
    pub fn validate(container: &Container) -> Result<(), Vec<AssetFileError>> {
        let mut errors = Vec::new();
        for asset in container.assets.values() {
            let path = container.base_path().join(asset.path.as_path());
            if let Err(err) = fs::canonicalize(path) {
                errors.push(AssetFileError {
                    asset: asset.rid.clone(),
                    kind: err.kind(),
                });
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

pub mod error {
    use serde::{Deserialize, Serialize};
    use std::io;
    use std::path::PathBuf;
    use thot_core::types::ResourceId;

    #[derive(Serialize, Deserialize, thiserror::Error, Debug)]
    pub enum Error {
        #[error("{0}")]
        Root(#[serde(with = "crate::error::IoErrorKind")] io::ErrorKind),

        #[error("container: {container:?}, assets: {assets:?}, settings: {settings:?}")]
        Properties {
            container: Option<Properties>,
            assets: Option<Properties>,
            settings: Option<Properties>,
        },
    }

    #[derive(Serialize, Deserialize, thiserror::Error, Debug)]
    pub enum Properties {
        #[error("{path:?} {kind:?}")]
        Io {
            path: PathBuf,

            #[serde(with = "crate::error::IoErrorKind")]
            kind: io::ErrorKind,
        },

        #[error("{path:?} {err:?}")]
        Serde { path: PathBuf, err: String },
    }

    #[derive(Serialize, Deserialize, thiserror::Error, Debug)]
    #[error("file for Asset {asset} {kind:?}")]
    pub struct AssetFile {
        pub(crate) asset: ResourceId,

        #[serde(with = "crate::error::IoErrorKind")]
        pub(crate) kind: io::ErrorKind,
    }
}
