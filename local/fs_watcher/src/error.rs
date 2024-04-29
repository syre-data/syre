use std::path::PathBuf;
use uuid::Uuid;

pub type Result<T = ()> = std::result::Result<T, Error>;

/// Errors representing a logical issue with processing or an unknown state has been entered.
#[derive(Debug)]
pub struct Error {
    /// Id of parent. Used for tracking errors across boundaries.
    parent: Option<Uuid>,
    pub kind: ErrorKind,
    pub paths: Vec<PathBuf>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            parent: None,
            kind,
            paths: Vec::new(),
        }
    }

    pub fn with_parent(kind: ErrorKind, parent: Uuid) -> Self {
        Self {
            parent: Some(parent),
            kind,
            paths: Vec::new(),
        }
    }

    pub fn parent(&self) -> &Option<Uuid> {
        &self.parent
    }

    pub fn add_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.paths.push(path.into());
        self
    }
}

#[derive(Debug, derive_more::From)]
pub enum ErrorKind {
    /// An event could not be fully processed.
    Conversion,

    /// An error occurred with the underlying watcher.
    Notify(notify::Error),
}

pub(crate) mod processing {
    pub type Result<T = ()> = std::result::Result<T, Error>;

    #[derive(Debug)]
    pub struct Error {
        pub kind: ErrorKind,
        pub description: String,
    }

    impl Error {
        pub fn new(kind: ErrorKind, description: impl Into<String>) -> Self {
            Self {
                kind,
                description: description.into(),
            }
        }
    }

    #[derive(Debug)]
    pub enum ErrorKind {
        /// A state was entered that should not be possible.
        State,

        /// An event caused something to happen that shouldn't have.
        Project,
    }
}

pub(crate) mod event {
    //! Event errors meant to be reported with events that caused them.
    use std::path::PathBuf;
    use syre_local::error::IoSerde;

    pub type Result<T = ()> = std::result::Result<T, Error>;

    #[derive(Debug)]
    pub struct Error {
        path: PathBuf,
        kind: ErrorKind,
    }

    impl Error {
        pub fn new(path: PathBuf, kind: ErrorKind) -> Self {
            Self { path, kind }
        }

        pub fn path(&self) -> &PathBuf {
            &self.path
        }

        pub fn kind(&self) -> &ErrorKind {
            &self.kind
        }
    }

    #[derive(Debug, PartialEq, derive_more::From)]
    pub enum ErrorKind {
        Resource(Resource),
        Project(Project),
    }

    #[derive(Debug, PartialEq)]
    pub enum Resource {
        PathNotInProject,
    }

    #[derive(Debug, PartialEq, derive_more::From)]
    pub enum Project {
        /// Project could no be properly loaded.
        Load(IoSerde),
    }
}
