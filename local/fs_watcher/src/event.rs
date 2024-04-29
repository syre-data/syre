pub(crate) mod file_system {
    //! File system events.
    use std::{path::PathBuf, time::Instant};
    use uuid::Uuid;

    #[derive(Debug)]
    pub struct Event {
        /// Id used to track the event across boundaries.
        id: Uuid,

        /// The instant the event was created.
        pub time: Instant,

        pub kind: EventKind,
    }

    impl Event {
        pub fn new(kind: impl Into<EventKind>, time: Instant) -> Self {
            Self {
                id: Uuid::now_v7(),
                time,
                kind: kind.into(),
            }
        }

        /// # Returns
        /// The unique id (uuid v7) assigned to the event.
        /// This is used to track the event across boundaries.
        pub fn id(&self) -> &Uuid {
            &self.id
        }
    }

    #[derive(Debug, derive_more::From)]
    pub enum EventKind {
        File(File),
        Folder(Folder),

        /// Could not determine if the event affect a file, folder, or other resource.
        Any(Any),
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
        Moved {
            from: PathBuf,
            to: PathBuf,
        },

        /// The content of the file changed.
        DataModified(PathBuf),

        /// The file was modified, but the type of change could not be determined.
        Other(PathBuf),
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

        /// The folder was modified, but the type of change could not be determined.
        Other(PathBuf),
    }

    #[derive(Debug)]
    pub enum Any {
        Removed(PathBuf),
    }
}

pub mod app {
    //! Syre application events.
    use crate::error::event::Error;
    use std::{path::PathBuf, time::Instant};
    use uuid::Uuid;

    #[derive(Debug)]
    pub struct Event {
        /// Unique id used to track events across boundaries.
        id: Uuid,

        /// Id of parent event.
        parent: Option<Uuid>,

        /// Instant the underlying event was created.
        time: Instant,

        kind: EventKind,

        paths: Vec<PathBuf>,

        /// Any errors that occurred during processing,
        /// so processing was not completed.
        errors: Vec<Error>,
    }

    impl Event {
        pub fn new(kind: EventKind) -> Self {
            Self {
                id: Uuid::now_v7(),
                parent: None,
                time: Instant::now(),
                kind,
                paths: Vec::new(),
                errors: Vec::new(),
            }
        }

        pub fn with_parent_and_time(kind: EventKind, parent: Uuid, time: Instant) -> Self {
            Self {
                id: Uuid::now_v7(),
                parent: Some(parent),
                time,
                kind,
                paths: Vec::new(),
                errors: Vec::new(),
            }
        }

        pub fn id(&self) -> &Uuid {
            &self.id
        }

        pub fn parent(&self) -> &Option<Uuid> {
            &self.parent
        }

        pub fn time(&self) -> &Instant {
            &self.time
        }

        pub fn kind(&self) -> &EventKind {
            &self.kind
        }

        pub fn paths(&self) -> &Vec<PathBuf> {
            &self.paths
        }

        pub fn errors(&self) -> &Vec<Error> {
            &self.errors
        }

        pub fn has_errors(&self) -> bool {
            !self.errors.is_empty()
        }

        /// Sets `paths` to a single path.
        pub fn add_path(mut self, path: PathBuf) -> Self {
            self.paths.push(path);
            self
        }

        pub fn add_error(mut self, error: Error) -> Self {
            self.errors.push(error);
            self
        }
    }

    #[derive(Debug, derive_more::From)]
    pub enum EventKind {
        /// An application config resource was modified.
        #[from]
        Config(Config),

        #[from]
        Project(Project),

        #[from]
        Graph(Graph),

        #[from]
        Container(Container),

        /// An asset file was modified.
        /// The file may not be assocated with an Asset yet.
        AssetFile(ResourceEvent),

        /// An analysis file was modified.
        /// The file may not be assocated with an Analysis yet.
        AnalysisFile(ResourceEvent),

        /// A file not associated to a resource type was modified.
        File(ResourceEvent),

        /// A folder not associated with a resource type was modified.
        ///
        /// This is the default event in case the resource type could not be determined.
        /// This occurs for instance, when a folder is removed.
        /// It is left to the client application to determine if the folder actually represented a
        /// resource.
        Folder(ResourceEvent),

        /// An unknown resource was modified.
        #[from]
        Any(Any),

        /// Indicates the provided events may not be in sync with the file system any longer.
        /// Programs relying on an in-memory representation of the file system should sync directly
        /// with the file system.
        /// See [`notify::event::Flag::Rescan] for more info.
        OutOfSync,
    }

    /// A resource that is distinguished by path.
    /// i.e. The resource's file is identified by its path.
    ///
    /// This means renaming the file at that path to another name
    /// effectively removes that file from begin identified as the resource,
    /// and renaming a file to that path effectively creates or modifies the resource.
    #[derive(Debug)]
    pub enum StaticResourceEvent {
        Created,
        Removed,

        /// The resource was modified.
        /// This could occur for multiple reasons including:
        /// + A file was moved to the resource's path.
        /// + The content of the resource's file was modified.
        Modified(ModifiedKind),

        /// The event caused the resource to change its type.
        ///
        /// # Examples
        /// + Renaming a config (.syre) folder causes it to be a normal folder.
        KindChanged,
    }

    /// A resource that is distinguished by its inode (macOS, *nix) or file id (Windows).
    /// i.e. The resource's file is the resource its self.
    #[derive(Debug)]
    pub enum ResourceEvent {
        Created,
        Removed,
        Renamed,

        /// The resource file was moved within the same project.
        Moved,

        /// The resource file was moved into another project.
        MovedProject,

        /// The resource was modified.
        /// This could occur for multiple reasons including:
        /// + A file was moved to the resource's path.
        /// + The content of the resource's file was modified.
        Modified(ModifiedKind),
    }

    #[derive(Debug)]
    pub enum ModifiedKind {
        /// The content of the resource was changed.
        Data,

        /// The type of modification could not be determined.
        /// The resource likely needs to be rescanned.
        Other,
    }

    /// App config events.
    #[derive(Debug)]
    pub enum Config {
        /// The app config directory was created.
        Created,

        /// The app config directory was removed.
        Removed,

        /// The app config directory was modified.
        Modified(ModifiedKind),

        /// The project manifest file was modified.
        ProjectManifest(StaticResourceEvent),

        /// The user manifest file was modified.
        UserManifest(StaticResourceEvent),
    }

    #[derive(Debug)]
    pub enum Project {
        /// The project was deleted.
        Removed,

        /// The project folder was moved or renamed.
        Moved,

        /// The project's config dir was modified.
        ConfigDir(StaticResourceEvent),

        /// The project's analysis dir was modified.
        AnalysisDir(ResourceEvent),

        /// The project's data dir was modified.
        DataDir(ResourceEvent),

        /// The project's properties file was modified.
        Properties(StaticResourceEvent),

        /// The project's settings file was modified.
        Settings(StaticResourceEvent),

        /// The project's analyses file was modified.
        Analysis(StaticResourceEvent),

        /// The project directory was modified.
        /// It should likely be rescanned.
        Modified,
    }

    /// Graph events.
    ///
    /// # Notes
    /// + The `Removed` event indicates that a folder that could be determined to be a container
    /// was removed. This can not always be determined, in which case a [Folder] event is emitted.
    #[derive(Debug)]
    pub enum Graph {
        /// A directory in a project's data folder was created as a container.
        Created,

        /// A container directory in a project's data folder was removed.
        Removed,

        /// A container directory was moved within the same project.
        Moved,

        /// A container directory was modified.
        Modified(ModifiedKind),
    }

    #[derive(Debug)]
    pub enum Container {
        /// The name of the container's folder was changed.
        Renamed,

        /// The container's config dir (i.e. `.syre` folder) was modified.
        ConfigDir(StaticResourceEvent),

        /// The container's properties file was modified.
        Properties(StaticResourceEvent),

        /// The container's settings file was modified.
        Settings(StaticResourceEvent),

        /// The container's assets file was modified.
        Assets(StaticResourceEvent),
    }

    #[derive(Debug)]
    pub enum Any {
        Removed,
    }
}
