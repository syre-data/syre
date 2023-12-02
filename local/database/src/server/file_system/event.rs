//! File system events

pub mod file_system {
    use std::path::PathBuf;

    #[derive(Debug)]
    pub enum Event {
        File(File),
        Folder(Folder),

        /// Could not determine if the event affect a file, folder, or other resource.
        Any(Any),
    }

    impl From<File> for Event {
        fn from(event: File) -> Self {
            Self::File(event)
        }
    }

    impl From<Folder> for Event {
        fn from(event: Folder) -> Self {
            Self::Folder(event)
        }
    }

    impl From<Any> for Event {
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
    }

    #[derive(Debug)]
    pub enum Any {
        Created(PathBuf),
        Removed(PathBuf),
    }
}

pub mod thot {
    use std::path::PathBuf;
    use thot_core::graph::ResourceTree;
    use thot_core::types::ResourceId;
    use thot_local::project::resources::Container as LocalContainer;

    #[derive(Debug)]
    pub enum Event {
        Graph(Graph),
        Container(Container),
        Asset(Asset),
        Script(Script),
        File(File),
        Folder(Folder),
    }

    impl From<Graph> for Event {
        fn from(event: Graph) -> Self {
            Self::Graph(event)
        }
    }

    impl From<Container> for Event {
        fn from(event: Container) -> Self {
            Self::Container(event)
        }
    }

    impl From<Asset> for Event {
        fn from(event: Asset) -> Self {
            Self::Asset(event)
        }
    }

    impl From<Script> for Event {
        fn from(event: Script) -> Self {
            Self::Script(event)
        }
    }

    impl From<File> for Event {
        fn from(event: File) -> Self {
            Self::File(event)
        }
    }

    impl From<Folder> for Event {
        fn from(event: Folder) -> Self {
            Self::Folder(event)
        }
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
