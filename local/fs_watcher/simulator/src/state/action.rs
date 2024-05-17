use super::{
    fs::{File, Folder},
    Ptr,
};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub enum Action {
    CreateFolder {
        parent: Ptr<Folder>,
        name: OsString,
    },

    CreateFolderAt {
        path: PathBuf,
        with_parents: bool,
    },

    CreateFile {
        parent: Ptr<Folder>,
        name: OsString,
    },

    CreateFileAt {
        path: PathBuf,
        with_parents: bool,
    },

    Remove(FsResource),

    Rename {
        resource: FsResource,
        to: PathBuf,
    },

    Move {
        resource: FsResource,
        parent: Ptr<Folder>,
    },

    Copy {
        resource: FsResource,
        parent: Ptr<Folder>,
    },

    Modify {
        file: Ptr<File>,
        kind: ModifyKind,
    },
}

#[derive(Debug, Clone, derive_more::From)]
pub enum FsResource {
    File(Ptr<File>),
    Folder(Ptr<Folder>),
}

#[derive(Debug)]
pub enum ModifyKind {
    /// Add an item to a manifest.
    ManifestAdd(String),

    /// Remove an item from a manifest.
    ManifestRemove(usize),
    Corrupt,
    Repair,
    Other,
}
