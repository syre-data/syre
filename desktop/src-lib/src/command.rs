//! Shared types between the desktop ui and tauri commands.
pub mod container {
    pub mod error {
        use serde::{Deserialize, Serialize};
        use std::io;
        use syre_local::error::{IoErrorKind, IoSerde};

        /// Error renaming container.
        #[derive(Serialize, Deserialize, Debug)]
        pub enum Rename {
            ProjectNotFound,
            NameCollision,
            Rename(#[serde(with = "IoErrorKind")] io::ErrorKind),
        }

        /// Error updating container.
        #[derive(Serialize, Deserialize, Debug)]
        pub enum Update {
            ProjectNotFound,
            Load(IoSerde),
            Save(#[serde(with = "IoErrorKind")] io::ErrorKind),
        }
    }
}

pub mod error {
    use serde::{Deserialize, Serialize};
    use std::io;

    /// Wrapper to allow for serialization
    #[derive(Serialize, Deserialize, derive_more::From)]
    pub struct IoErrorKind(#[serde(with = "syre_local::error::IoErrorKind")] pub io::ErrorKind);
}
