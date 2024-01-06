pub mod container {
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

pub mod tree {
    use super::container::Error as LoaderError;
    use serde::{Deserialize, Serialize};
    use std::io;

    #[derive(Serialize, Deserialize, thiserror::Error, Debug)]
    pub enum Error {
        #[error("{0}")]
        Dir(#[serde(with = "crate::error::IoErrorKind")] io::ErrorKind),

        #[error("{0}")]
        Load(LoaderError),
    }
}
