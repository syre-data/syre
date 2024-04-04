//! Errors
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::result::Result as StdResult;
use syre_core::types::ResourceId;
use syre_core::Error as CoreError;
use syre_local::error::{Error as Local, IoSerde};
use syre_local::loader::error::container::AssetFile;
use syre_local::loader::error::container::Error as ContainerLoader;
use syre_local::loader::error::tree::Error as ContainerTreeLoader;
use thiserror::Error;

#[cfg(feature = "server")]
use crate::types::SocketType;

type ContainerTree = syre_core::graph::ResourceTree<syre_core::project::Container>;

/// [`Database`](crate::db) related errors.
#[derive(Serialize, Deserialize, Error, Debug)]
pub enum Error {
    /// A ZMQ [`Context`](zmq::Context) does not exist where expected.
    #[error("ZMQ context does not exist")]
    ContextDoesNotExist,

    /// A ZMQ error.
    #[error("{0}")]
    ZMQ(String),

    /// A type of socket is required, but has not yet been created.
    #[cfg(feature = "server")]
    #[error("{0:?}")]
    SocketDoesNotExist(SocketType),

    #[cfg(feature = "server")]
    #[error("{0}")]
    SettingsError(String),

    #[error("{0:?}")]
    Core(CoreError),

    #[error("{0}")]
    Local(Local),

    #[error("{0}")]
    TrashError(String),

    /// Issue with the database.
    #[error("{0}")]
    Database(String),

    /// The database has become out of sync.
    #[error("database out of sync")]
    OutOfSync,

    #[error("{0}")]
    IoSerde(IoSerde),

    #[error("{0}")]
    LoadContainer(ContainerLoader),

    #[error("{0:?}")]
    LoadTree(HashMap<PathBuf, ContainerTreeLoader>),

    #[error("{errors:?}")]
    LoadPartial {
        errors: HashMap<PathBuf, ContainerTreeLoader>,
        graph: Option<ContainerTree>,
    },

    #[error("{errors:?}")]
    AssetValidation {
        errors: HashMap<ResourceId, Vec<AssetFile>>,
        graph: ContainerTree,
    },

    #[cfg(feature = "server")]
    #[error("{0}")]
    Update(server::Update),

    #[cfg(feature = "server")]
    #[error("{0}")]
    UpdateContainer(server::UpdateContainer),

    #[cfg(feature = "server")]
    #[error("{0}")]
    LoadProjectGraph(server::LoadProjectGraph),
}

#[cfg(any(feature = "server", feature = "client"))]
impl From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Self {
        Self::ZMQ(err.message().to_string())
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

impl From<Local> for Error {
    fn from(err: Local) -> Self {
        Self::Local(err)
    }
}

impl From<IoSerde> for Error {
    fn from(value: IoSerde) -> Self {
        Self::IoSerde(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Local(value.into())
    }
}

#[cfg(feature = "server")]
impl From<ContainerLoader> for Error {
    fn from(value: ContainerLoader) -> Self {
        Self::LoadContainer(value)
    }
}

#[cfg(feature = "server")]
impl From<HashMap<PathBuf, ContainerTreeLoader>> for Error {
    fn from(value: HashMap<PathBuf, ContainerTreeLoader>) -> Self {
        Self::LoadTree(value)
    }
}

#[cfg(feature = "server")]
impl From<trash::Error> for Error {
    fn from(err: trash::Error) -> Self {
        Error::TrashError(format!("{err:?}"))
    }
}

#[cfg(feature = "server")]
impl From<server::Update> for Error {
    fn from(value: server::Update) -> Self {
        Self::Update(value)
    }
}

#[cfg(feature = "server")]
impl From<server::UpdateContainer> for Error {
    fn from(value: server::UpdateContainer) -> Self {
        Self::UpdateContainer(value)
    }
}

#[cfg(feature = "server")]
impl From<server::LoadProjectGraph> for Error {
    fn from(value: server::LoadProjectGraph) -> Self {
        Self::LoadProjectGraph(value)
    }
}

pub type Result<T = ()> = StdResult<T, Error>;

pub mod server {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::io;
    use std::path::PathBuf;
    use syre_core::error::Project as ProjectError;
    use syre_core::project::Project;
    use syre_core::types::ResourceId;
    use syre_local::error::IoErrorKind;
    use syre_local::error::IoSerde;
    use syre_local::types::ProjectSettings;
    use thiserror::Error;

    type CoreContainerTree = syre_core::graph::ResourceTree<syre_core::project::Container>;

    #[derive(Serialize, Deserialize, Error, Debug)]
    pub enum LoadUserProjects {
        #[error("could not load project manifest: {0}")]
        LoadProjectsManifest(IoSerde),

        #[error("{errors:?}")]
        LoadProjects {
            projects: Vec<(Project, ProjectSettings)>,
            errors: HashMap<PathBuf, IoSerde>,
        },
    }

    #[serde_with::serde_as]
    #[derive(Serialize, Deserialize, Clone, Error, Debug)]
    pub enum LoadProjectGraph {
        #[error("project not found")]
        ProjectNotFound,

        #[error("{0:?}")]
        Project(ProjectError),

        #[error("{errors:?}")]
        Load {
            errors: HashMap<PathBuf, syre_local::loader::error::tree::Error>,
            graph: Option<CoreContainerTree>,
        },

        #[error("{0:?}")]
        InsertContainers(
            #[serde_as(as = "HashMap<_, syre_local::error::IoErrorKind>")]
            HashMap<PathBuf, io::ErrorKind>,
        ),

        #[error("{errors:?}")]
        InsertAssets {
            #[serde_as(as = "HashMap<_, syre_local::error::IoErrorKind>")]
            errors: HashMap<ResourceId, io::ErrorKind>,
            graph: CoreContainerTree,
        },
    }

    #[derive(Serialize, Deserialize, Error, Debug)]
    pub enum Update {
        #[error("resource not found")]
        ResourceNotFound,

        #[error("{0}")]
        Save(IoSerde),
    }

    impl From<IoSerde> for Update {
        fn from(value: IoSerde) -> Self {
            Self::Save(value)
        }
    }

    #[derive(Serialize, Deserialize, Error, Debug)]
    pub enum Rename {
        #[error("resource not found")]
        ResourceNotFound,

        #[error("name conflict")]
        NameConflict,

        #[error("{0}")]
        Rename(#[serde(with = "IoErrorKind")] io::ErrorKind),

        #[error("{0}")]
        UpdateSubgraphPath(UpdateSubgraphPath),
    }

    impl From<UpdateSubgraphPath> for Rename {
        fn from(value: UpdateSubgraphPath) -> Self {
            Self::UpdateSubgraphPath(value)
        }
    }

    #[derive(Error, Debug, Serialize, Deserialize)]
    pub enum UpdateSubgraphPath {
        #[error("subgraph root not found")]
        RootNotFound,

        #[error("could not canonicalize resource {resource} with path {path}: {kind}")]
        Canonicalization {
            resource: ResourceId,
            path: PathBuf,

            #[serde(with = "IoErrorKind")]
            kind: io::ErrorKind,
        },
    }

    #[derive(Serialize, Deserialize, Error, Debug)]
    pub enum UpdateContainer {
        #[error("resource not found")]
        ResourceNotFound,

        #[error("{0}")]
        Rename(Rename),

        #[error("{0}")]
        Save(IoSerde),
    }

    impl<T> From<T> for UpdateContainer
    where
        T: Into<Rename>,
    {
        fn from(value: T) -> Self {
            Self::Rename(value.into())
        }
    }
}
