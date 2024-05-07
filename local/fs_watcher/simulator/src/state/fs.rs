use std::{
    cell::RefCell,
    path::PathBuf,
    rc::{Rc, Weak},
};

pub type Node<T> = Rc<RefCell<T>>;
pub type NodeRef<T> = Weak<RefCell<T>>;

pub struct Folder {
    pub name: PathBuf,
    pub resource: FolderResource,
    pub parent: Option<NodeRef<Folder>>,
    pub children: Vec<Node<Folder>>,
    pub files: Vec<Node<File>>,
}

pub struct File {
    pub name: PathBuf,
    pub resource: FileResource,
    pub parent: NodeRef<Folder>,
}

pub enum FolderResource {
    Project,
    ProjectConfig,
    ProjectAnalyses,
    ProjectData,
    Container,
    ContainerConfig,
}

pub enum FileResource {
    UserManifest,
    ProjectManifest,
    ProjectProperties,
    ProjectSettings,
    ProjectAnalyses,
    ContainerProperties,
    ContainerSettings,
    ContainerAssets,
    AssetFile,
}
