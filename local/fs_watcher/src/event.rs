//! File system events.

pub(crate) mod file_system {
    //! File system events.
    use std::path::PathBuf;
    use std::time::Instant;
    use uuid::Uuid;

    #[derive(Debug)]
    pub struct Event {
        /// Id used to track the event across boundaries.
        event_id: Uuid,

        pub kind: EventKind,

        /// The instant the event was created.
        pub time: Instant,
    }

    impl Event {
        pub fn new(kind: impl Into<EventKind>, time: Instant) -> Self {
            Self {
                event_id: Uuid::now_v7(),
                kind: kind.into(),
                time,
            }
        }

        /// # Returns
        /// The unique id (uuid v7) assigned to the event.
        /// This is used to track the event across boundaries.
        pub fn event_id(&self) -> &Uuid {
            &self.event_id
        }
    }

    #[derive(Debug)]
    pub enum EventKind {
        File(File),
        Folder(Folder),

        /// Could not determine if the event affect a file, folder, or other resource.
        Any(Any),

        /// Indicates the provided events may not be in sync with the file system any longer.
        /// Programs relying on an in-memory representation of the file system should sync directly
        /// with the file system.
        /// See [`notify::event::Flag::Rescan] for more info.
        OutOfSync,
    }

    impl From<File> for EventKind {
        fn from(event: File) -> Self {
            Self::File(event)
        }
    }

    impl From<Folder> for EventKind {
        fn from(event: Folder) -> Self {
            Self::Folder(event)
        }
    }

    impl From<Any> for EventKind {
        fn from(event: Any) -> Self {
            Self::Any(event)
        }
    }

    #[derive(Debug)]
    pub enum File {
        Created(PathBuf),
        Removed(PathBuf),

        /// A file's name was changed.
        /// Its base directory is unchanged.
        Renamed {
            from: PathBuf,
            to: PathBuf,
        },

        /// A file was moved to a different folder.
        /// Its file name is unchanged.
        Moved {
            from: PathBuf,
            to: PathBuf,
        },

        Modified(PathBuf),
    }

    #[derive(Debug)]
    pub enum Folder {
        /// A new folder was created.
        /// This folder may already have contents in it, e.g. if it was pasted in from another location.
        Created(PathBuf),

        Removed(PathBuf),

        /// A folder's name was changed.
        Renamed {
            from: PathBuf,
            to: PathBuf,
        },

        /// A folder was moved to a different parent.
        Moved {
            from: PathBuf,
            to: PathBuf,
        },

        Modified(PathBuf),
    }

    #[derive(Debug)]
    pub enum Any {
        Removed(PathBuf),
    }
}

pub mod app {
    //! Syre events.
    use std::path::PathBuf;
    use syre_core::graph::ResourceTree;
    use syre_core::types::ResourceId;
    use syre_local::project::resources::Container as LocalContainer;
    use uuid::Uuid;

    #[derive(Debug)]
    pub struct Event {
        event_id: Uuid,
        kind: EventKind,
    }

    impl Event {
        pub fn with_id(event_id: Uuid, kind: EventKind) -> Self {
            Self { event_id, kind }
        }

        pub fn event_id(&self) -> &Uuid {
            &self.event_id
        }

        pub fn kind(&self) -> &EventKind {
            &self.kind
        }
    }

    #[derive(Debug)]
    pub enum EventKind {
        Project(Project),
        Graph(Graph),
        Container(Container),
        Asset(Asset),
        Script(Script),
        File(File),
        Folder(Folder),
    }

    impl From<Project> for EventKind {
        fn from(event: Project) -> Self {
            Self::Project(event)
        }
    }

    impl From<Graph> for EventKind {
        fn from(event: Graph) -> Self {
            Self::Graph(event)
        }
    }

    impl From<Container> for EventKind {
        fn from(event: Container) -> Self {
            Self::Container(event)
        }
    }

    impl From<Asset> for EventKind {
        fn from(event: Asset) -> Self {
            Self::Asset(event)
        }
    }

    impl From<Script> for EventKind {
        fn from(event: Script) -> Self {
            Self::Script(event)
        }
    }

    impl From<File> for EventKind {
        fn from(event: File) -> Self {
            Self::File(event)
        }
    }

    impl From<Folder> for EventKind {
        fn from(event: Folder) -> Self {
            Self::Folder(event)
        }
    }

    #[derive(Debug)]
    pub enum Project {
        /// The project was deleted.
        Removed(ResourceId),
        Moved {
            project: ResourceId,
            path: PathBuf,
        },
    }

    #[derive(Debug)]
    pub enum Graph {
        /// An existing graph was inserted.
        /// The graph could not be loaded from the database.
        Inserted(ResourceTree<LocalContainer>),

        /// An existing graph was copied.
        Copied(ResourceTree<LocalContainer>),

        Removed(ResourceId),

        /// A subgraph was moved.
        Moved {
            root: ResourceId,
            path: PathBuf,
        },
    }

    #[derive(Debug)]
    pub enum Container {
        /// The name of the `Container`'s folder was changed.
        Renamed {
            container: ResourceId,
            name: PathBuf,
        },
    }

    #[derive(Debug)]
    pub enum Asset {
        Removed(ResourceId),
        Moved {
            asset: ResourceId,
            path: PathBuf,
        },

        /// The name of the `Asset`'s file was changed.
        Renamed {
            asset: ResourceId,
            name: PathBuf,
        },

        /// A file corresponding to a registered `Asset` was created.
        FileCreated(ResourceId),
    }

    #[derive(Debug)]
    pub enum Script {
        /// A `Script`` was created.
        Created(PathBuf),
        Removed(ResourceId),

        /// A `Script`'s path changed.
        ///
        /// # Notes
        /// + The `Script` may have been moved into a different `Project`.
        Moved {
            script: ResourceId,
            path: PathBuf,
        },
    }

    #[derive(Debug)]
    pub enum Folder {
        Created(PathBuf),
    }

    #[derive(Debug)]
    pub enum File {
        Created(PathBuf),
    }
}
