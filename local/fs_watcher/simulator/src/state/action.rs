use super::{
    fs::{File, Folder},
    Ptr,
};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub enum Action {
    CreateFolder { path: PathBuf, with_parents: bool },
    CreateFile { path: PathBuf, with_parents: bool },
    Remove(PathBuf),
    Rename { from: PathBuf, to: OsString },
    Move { from: PathBuf, to: PathBuf },
    Copy { from: PathBuf, to: PathBuf },
    Modify { file: PathBuf, kind: ModifyKind },
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
