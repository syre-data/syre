//! Shared types between the desktop ui and tauri commands.
pub mod container {
    pub mod error {
        use serde::{Deserialize, Serialize};
        use std::io;
        use syre_local::error::IoErrorKind;

        /// Error renaming container.
        #[derive(Serialize, Deserialize, Debug)]
        pub enum Rename {
            ProjectNotFound,
            PropertiesNotOk,
            NameCollision,
            Io(#[serde(with = "IoErrorKind")] io::ErrorKind),
        }
    }
}
