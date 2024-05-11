use std::path::PathBuf;

#[derive(Debug)]
pub struct Action {
    resource: Resource,
    action: ResourceAction,
}

impl Action {
    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    pub fn action(&self) -> &ResourceAction {
        &self.action
    }
}

impl Action {
    pub fn new(resource: Resource, action: ResourceAction) -> Self {
        Self { resource, action }
    }

    pub fn file_create(path: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::File,
            action: ResourceAction::Create(path.into()),
        }
    }

    pub fn file_remove(path: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::File,
            action: ResourceAction::Remove(path.into()),
        }
    }

    pub fn file_rename(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::File,
            action: ResourceAction::Rename {
                from: from.into(),
                to: to.into(),
            },
        }
    }

    pub fn file_move(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::File,
            action: ResourceAction::Move {
                from: from.into(),
                to: to.into(),
            },
        }
    }

    pub fn file_copy(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::File,
            action: ResourceAction::Copy {
                from: from.into(),
                to: to.into(),
            },
        }
    }

    pub fn folder_create(path: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::Folder,
            action: ResourceAction::Create(path.into()),
        }
    }

    pub fn folder_remove(path: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::Folder,
            action: ResourceAction::Remove(path.into()),
        }
    }

    pub fn folder_rename(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::Folder,
            action: ResourceAction::Rename {
                from: from.into(),
                to: to.into(),
            },
        }
    }

    pub fn folder_move(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::Folder,
            action: ResourceAction::Move {
                from: from.into(),
                to: to.into(),
            },
        }
    }

    pub fn folder_copy(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::Folder,
            action: ResourceAction::Copy {
                from: from.into(),
                to: to.into(),
            },
        }
    }
}

#[derive(Debug)]
pub enum Resource {
    File,
    Folder,
}

#[derive(Debug)]
pub enum ResourceAction {
    Create(PathBuf),
    Remove(PathBuf),

    /// Rename a resource.
    ///
    /// # Fields
    /// + `from` should be an absolute path.
    /// + `to` should be a file name only.
    Rename {
        from: PathBuf,
        to: PathBuf,
    },

    /// Move a resource.
    ///
    /// # Fields
    /// + `from` should be an absolute path.
    /// + `to` should be an Absolute path.
    Move {
        from: PathBuf,
        to: PathBuf,
    },

    /// Copy a resource.
    ///
    /// # Fields
    /// + `from` should be an absolute path.
    /// + `to` should be an Absolute path.
    Copy {
        from: PathBuf,
        to: PathBuf,
    },
}
