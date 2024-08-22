//! Shared types between the desktop ui and tauri commands.
pub mod container {
    pub mod bulk {
        use super::super::{
            bulk::{MetadataAction, TagsAction},
            serde_opt_opt_str,
        };
        use serde::{Deserialize, Serialize};
        use syre_core::types::Value;

        #[derive(Serialize, Deserialize, Default, Debug)]
        pub struct PropertiesUpdate {
            pub name: Option<String>,

            #[serde(with = "serde_opt_opt_str")]
            pub kind: Option<Option<String>>,

            #[serde(with = "serde_opt_opt_str")]
            pub description: Option<Option<String>>,
            pub tags: TagsAction,
            pub metadata: MetadataAction,
        }

        pub mod error {
            use serde::{Deserialize, Serialize};
            use std::{io, path::PathBuf};
            use syre_local::error::{IoErrorKind, IoSerde};

            /// Error renaming container.
            #[derive(Serialize, Deserialize, Debug)]
            pub enum Rename {
                ProjectNotFound,

                /// Renaming would cause a name collision.
                NameCollision(Vec<PathBuf>),
            }

            /// Error updating containers.
            #[derive(Serialize, Deserialize, Debug)]
            pub enum Update {
                Load(IoSerde),
                Save(#[serde(with = "IoErrorKind")] io::ErrorKind),
            }
        }
    }

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

pub mod asset {
    pub mod bulk {
        use std::path::PathBuf;

        use super::super::{
            bulk::{MetadataAction, TagsAction},
            serde_opt_opt_str,
        };
        use serde::{Deserialize, Serialize};
        use syre_core::types::{ResourceId, Value};

        #[derive(Serialize, Deserialize, Default, Debug)]
        pub struct ContainerAssets {
            pub container: PathBuf,
            pub assets: Vec<ResourceId>,
        }

        impl From<(PathBuf, Vec<ResourceId>)> for ContainerAssets {
            fn from((container, assets): (PathBuf, Vec<ResourceId>)) -> Self {
                Self { container, assets }
            }
        }

        #[derive(Serialize, Deserialize, Default, Debug)]
        pub struct PropertiesUpdate {
            #[serde(with = "serde_opt_opt_str")]
            pub name: Option<Option<String>>,

            #[serde(with = "serde_opt_opt_str")]
            pub kind: Option<Option<String>>,

            #[serde(with = "serde_opt_opt_str")]
            pub description: Option<Option<String>>,
            pub tags: TagsAction,
            pub metadata: MetadataAction,
        }

        pub mod error {
            use serde::{Deserialize, Serialize};
            use std::{io, path::PathBuf};
            use syre_core::types::ResourceId;
            use syre_local::error::{IoErrorKind, IoSerde};

            /// Error updating containers.
            #[derive(Serialize, Deserialize, Debug)]
            pub enum Update {
                Load(IoSerde),
                NotFound(Vec<ResourceId>),
                Save(#[serde(with = "IoErrorKind")] io::ErrorKind),
            }
        }
    }

    pub mod error {
        use serde::{Deserialize, Serialize};
        use std::io;
        use syre_local::error::{IoErrorKind, IoSerde};

        /// Error updating asset.
        #[derive(Serialize, Deserialize, Debug)]
        pub enum Update {
            ProjectNotFound,
            Load(IoSerde),
            Save(#[serde(with = "IoErrorKind")] io::ErrorKind),
        }
    }
}

pub mod bulk {
    use super::serde_opt_opt_str;
    use serde::{Deserialize, Serialize};
    use syre_core::types::Value;

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct TagsAction {
        pub insert: Vec<String>,
        pub remove: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct MetadataAction {
        /// Values to insert new or update.
        pub insert: Vec<(String, Value)>,

        /// Keys to remove.
        pub remove: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct PropertiesUpdate {
        #[serde(with = "serde_opt_opt_str")]
        pub kind: Option<Option<String>>,

        #[serde(with = "serde_opt_opt_str")]
        pub description: Option<Option<String>>,
        pub tags: TagsAction,
        pub metadata: MetadataAction,
    }
}

pub mod error {
    use serde::{Deserialize, Serialize};
    use std::io;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ProjectNotFound;

    /// Wrapper to allow for serialization
    #[derive(Serialize, Deserialize, derive_more::From, Debug)]
    pub struct IoErrorKind(#[serde(with = "syre_local::error::IoErrorKind")] pub io::ErrorKind);
    impl Into<io::ErrorKind> for IoErrorKind {
        fn into(self) -> io::ErrorKind {
            self.0
        }
    }
    impl From<io::Error> for IoErrorKind {
        fn from(value: io::Error) -> Self {
            Self(value.kind())
        }
    }
}

mod serde_opt_opt_str {
    use serde::{de, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Option<String>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(opt_str) => match opt_str {
                None => serializer.serialize_str(""),
                Some(val) => serializer.serialize_str(&val),
            },
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = OptOptStrVisitor {};
        deserializer.deserialize_option(visitor)
    }

    struct OptOptStrVisitor;
    impl<'de> de::Visitor<'de> for OptOptStrVisitor {
        type Value = Option<Option<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.is_empty() {
                Ok(Some(None))
            } else {
                Ok(Some(Some(v.to_string())))
            }
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let visitor = OptOptStrVisitor {};
            deserializer.deserialize_str(visitor)
        }
    }
}
