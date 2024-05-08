use std::{
    cell::RefCell,
    path::PathBuf,
    rc::{Rc, Weak},
};

pub type Node<T> = Rc<RefCell<T>>;
pub type NodeRef<T> = Weak<RefCell<T>>;

#[derive(Default, Debug)]
pub struct State {
    roots: Vec<Node<Folder>>,
    folders: Vec<NodeRef<Folder>>,
    files: Vec<NodeRef<File>>,
}

impl State {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn folders(&self) -> Vec<NodeRef<Folder>> {
        self.folders.clone()
    }

    pub fn files(&self) -> Vec<NodeRef<File>> {
        self.files.clone()
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        let roots = self
            .roots
            .iter()
            .map(|root| Rc::new(RefCell::new(root.borrow().duplicate())))
            .collect::<Vec<_>>();

        let mut folders = Vec::with_capacity(self.folders.len());
        for root in roots.iter() {
            folders.extend(root.borrow().descendants());
            folders.push(Rc::downgrade(&root));
        }

        let files = folders
            .iter()
            .flat_map(|folder| {
                let folder = folder.upgrade().unwrap();
                let folder = folder.borrow();
                folder
                    .files
                    .iter()
                    .map(|file| Rc::downgrade(file))
                    .collect::<Vec<_>>()
            })
            .collect();

        Self {
            roots,
            folders,
            files,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Folder {
    pub name: PathBuf,
    pub resource: Option<FolderResource>,
    pub parent: Option<NodeRef<Folder>>,
    pub children: Vec<Node<Folder>>,
    pub files: Vec<Node<File>>,
}

impl Folder {
    pub fn new(name: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            resource: None,
            parent: None,
            children: vec![],
            files: vec![],
        }
    }
}

impl Folder {
    pub fn duplicate(&self) -> Self {
        let children = self
            .children
            .iter()
            .map(|child| {
                let child = child.borrow();
                Rc::new(RefCell::new(child.duplicate()))
            })
            .collect();

        Self {
            name: self.name.clone(),
            resource: self.resource.clone(),
            parent: None,
            children,
            files: self.files.clone(),
        }
    }

    /// Returns a flat list of all descendant folders.
    /// Does not include self.
    pub fn descendants(&self) -> Vec<NodeRef<Self>> {
        self.children
            .iter()
            .flat_map(|child| {
                let mut flat = child.borrow().descendants();
                flat.push(Rc::downgrade(child));
                flat
            })
            .collect()
    }

    // /// Returns ancestors, not including self.
    // pub fn ancestors(&self) -> Vec<NodeRef<Self>> {
    //     let Some(mut parent) = self.parent.as_ref() else {
    //         return vec![];
    //     };

    //     let mut ancestors = vec![parent.clone()];
    //     let parent = parent.upgrade().unwrap();
    //     let mut child = parent.borrow();
    //     while let Some(parent) = child.parent.as_ref() {
    //         ancestors.push(parent.clone());
    //         child = parent.upgrade().unwrap();
    //         child = child.borrow();
    //     }

    //     ancestors
    // }
}

#[derive(Debug, Clone)]
pub struct File {
    pub name: PathBuf,
    pub resource: Option<FileResource>,
    pub parent: NodeRef<Folder>,
}

#[derive(Debug, Clone)]
pub enum FolderResource {
    Project,
    ProjectConfig,
    ProjectAnalyses,
    ProjectData,
    Container,
    ContainerConfig,
}

#[derive(Debug, Clone)]
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

#[derive(Clone, Debug, derive_more::From)]
pub enum Action {
    Folder(FolderAction),
    File(FileAction),
}

#[derive(Debug, Clone)]
pub enum FolderAction {
    /// Insert the folder.
    /// If `parent` is `None` create a new folder at root.
    Insert {
        folder: Folder,
        parent: Option<NodeRef<Folder>>,
    },

    /// Remove the folder and all its descendants.
    Remove(NodeRef<Folder>),

    /// Rename the folder.
    Rename {
        folder: NodeRef<Folder>,
        name: PathBuf,
    },

    /// Move into the given parent.
    /// If `parent` is `None` move to root.
    Move {
        folder: NodeRef<Folder>,
        parent: Option<NodeRef<Folder>>,
    },
}

#[derive(Debug, Clone)]
pub enum FileAction {
    /// Insert a file into a folder.
    Insert {
        name: PathBuf,
        resource: Option<FileResource>,
        parent: NodeRef<Folder>,
    },

    /// Remove the file.
    Remove(NodeRef<File>),

    /// Rename the file.
    Rename { file: NodeRef<File>, name: PathBuf },

    /// Move into the given parent.
    Move {
        file: NodeRef<File>,
        parent: NodeRef<Folder>,
    },
}

impl State {
    fn insert_folder(
        &mut self,
        folder: Folder,
        parent: Option<NodeRef<Folder>>,
    ) -> Result<(), error::Error> {
        match parent {
            None => {
                let folder = Rc::new(RefCell::new(folder));
                self.folders.push(Rc::downgrade(&folder));
                self.roots.push(folder);
                Ok(())
            }

            Some(parent) => {
                if !self.has_folder(&parent) {
                    return Err(error::Error::NotFound);
                };

                let parent = parent.upgrade().unwrap();
                let mut parent = parent.borrow_mut();
                if parent.children.iter().any(|child| {
                    let child = child.borrow();
                    child.name == folder.name
                }) {
                    return Err(error::Error::NameCollision);
                }

                let folder = Rc::new(RefCell::new(folder));
                self.folders.push(Rc::downgrade(&folder));
                parent.children.push(folder);
                Ok(())
            }
        }
    }

    fn remove_folder(&mut self, folder: NodeRef<Folder>) -> Result<(), error::Error> {
        if !self.has_folder(&folder) {
            return Err(error::Error::NotFound);
        };

        let folder_ptr = folder.upgrade().unwrap();
        for child in folder_ptr.borrow().children.iter() {
            self.remove_folder(Rc::downgrade(child));
        }

        self.folders.retain(|f| !f.ptr_eq(&folder));
        let folder = folder_ptr.borrow();
        self.files.retain(|r| {
            !folder.files.iter().any(|file| {
                let file = Rc::downgrade(&file);
                r.ptr_eq(&file)
            })
        });

        if let Some(parent) = folder.parent.as_ref() {
            let parent = parent.upgrade().unwrap();
            parent
                .borrow_mut()
                .children
                .retain(|child| !Rc::ptr_eq(child, &folder_ptr));
        }

        Ok(())
    }

    fn rename_folder(
        &mut self,
        folder: NodeRef<Folder>,
        name: impl Into<PathBuf>,
    ) -> Result<(), error::Error> {
        if !self.has_folder(&folder) {
            return Err(error::Error::NotFound);
        };

        let name = name.into();
        let folder_ptr = folder.upgrade().unwrap();
        let folder = folder_ptr.borrow();
        if let Some(parent) = folder.parent.as_ref() {
            let parent = parent.upgrade().unwrap();
            if parent.borrow().children.iter().any(|child| {
                let child = child.borrow();
                child.name == name
            }) {
                return Err(error::Error::NameCollision);
            }
        }

        let mut folder = folder_ptr.borrow_mut();
        folder.name = name;
        Ok(())
    }

    fn move_folder(
        &mut self,
        folder: NodeRef<Folder>,
        parent: Option<NodeRef<Folder>>,
    ) -> Result<(), error::Error> {
        if !self.has_folder(&folder) {
            return Err(error::Error::NotFound);
        };

        let folder_ptr = folder.upgrade().unwrap();
        let folder = folder_ptr.borrow();

        match (folder.parent.as_ref(), parent) {
            (None, None) => {}
            (Some(from_parent), None) => {
                let parent = from_parent.upgrade().unwrap();
                parent
                    .borrow_mut()
                    .children
                    .retain(|child| Rc::ptr_eq(child, &folder_ptr));

                self.roots.push(folder_ptr.clone());
            }

            (None, Some(to_parent)) => {
                if !self.has_folder(&to_parent) {
                    return Err(error::Error::NotFound);
                };

                if self.folder_has_name(&to_parent, &folder.name).unwrap() {
                    return Err(error::Error::NameCollision);
                }

                self.roots.retain(|root| !Rc::ptr_eq(root, &folder_ptr));
                let parent = to_parent.upgrade().unwrap();
                parent.borrow_mut().children.push(folder_ptr.clone());
            }

            (Some(from_parent), Some(to_parent)) => {
                if from_parent.ptr_eq(&to_parent) {
                    return Ok(());
                }

                if !self.has_folder(&to_parent) {
                    return Err(error::Error::NotFound);
                };

                if self.folder_has_name(&to_parent, &folder.name)? {
                    return Err(error::Error::NameCollision);
                }

                let parent = from_parent.upgrade().unwrap();
                parent
                    .borrow_mut()
                    .children
                    .retain(|child| Rc::ptr_eq(child, &folder_ptr));

                let parent = to_parent.upgrade().unwrap();
                parent.borrow_mut().children.push(folder_ptr.clone());
            }
        }

        Ok(())
    }
}

impl State {
    fn insert_file(
        &mut self,
        name: impl Into<PathBuf>,
        resource: Option<FileResource>,
        parent: NodeRef<Folder>,
    ) -> Result<(), error::Error> {
        if !self.has_folder(&parent) {
            return Err(error::Error::NotFound);
        };

        let file = Rc::new(RefCell::new(File {
            name: name.into(),
            resource,
            parent: parent.clone(),
        }));

        let parent_ptr = parent.upgrade().unwrap();
        self.files.push(Rc::downgrade(&file));
        parent_ptr.borrow_mut().files.push(file);
        Ok(())
    }

    fn remove_file(&mut self, file: NodeRef<File>) -> Result<(), error::Error> {
        if !self.has_file(&file) {
            return Err(error::Error::NotFound);
        }

        let file_ptr = file.upgrade().unwrap();
        let parent = file_ptr.borrow().parent.upgrade().unwrap();
        self.files.retain(|f| !f.ptr_eq(&file));
        parent
            .borrow_mut()
            .files
            .retain(|f| !Rc::ptr_eq(&file_ptr, f));

        Ok(())
    }

    fn rename_file(
        &mut self,
        file: NodeRef<File>,
        name: impl Into<PathBuf>,
    ) -> Result<(), error::Error> {
        if !self.has_file(&file) {
            return Err(error::Error::NotFound);
        }

        let name = name.into();
        let file_ptr = file.upgrade().unwrap();
        if self
            .folder_has_name(&file_ptr.borrow().parent, &name)
            .unwrap()
        {
            return Err(error::Error::NameCollision);
        }

        file_ptr.borrow_mut().name = name;
        Ok(())
    }

    fn move_file(
        &mut self,
        file: NodeRef<File>,
        parent: NodeRef<Folder>,
    ) -> Result<(), error::Error> {
        if !self.has_file(&file) {
            return Err(error::Error::NotFound);
        }

        if !self.has_folder(&parent) {
            return Err(error::Error::NotFound);
        };

        let file_ptr = file.upgrade().unwrap();
        let file = file_ptr.borrow();
        let from_parent = file.parent.upgrade().unwrap();
        from_parent
            .borrow_mut()
            .files
            .retain(|f| !Rc::ptr_eq(f, &file_ptr));

        let parent = parent.upgrade().unwrap();
        parent.borrow_mut().files.push(file_ptr.clone());
        Ok(())
    }
}

impl State {
    pub fn transition(&mut self, action: Action) -> Result<(), error::Error> {
        match action {
            Action::Folder(action) => self.handle_folder_action(action),
            Action::File(action) => self.handle_file_action(action),
        }
    }

    fn handle_folder_action(&mut self, action: FolderAction) -> Result<(), error::Error> {
        match action {
            FolderAction::Insert { folder, parent } => self.insert_folder(folder, parent),
            FolderAction::Remove(folder) => self.remove_folder(folder),
            FolderAction::Rename { folder, name } => self.rename_folder(folder, name),
            FolderAction::Move { folder, parent } => self.move_folder(folder, parent),
        }
    }

    fn handle_file_action(&mut self, action: FileAction) -> Result<(), error::Error> {
        match action {
            FileAction::Insert {
                name,
                resource,
                parent,
            } => self.insert_file(name, resource, parent),

            FileAction::Remove(file) => self.remove_file(file),
            FileAction::Rename { file, name } => self.rename_file(file, name),
            FileAction::Move { file, parent } => self.move_file(file, parent),
        }
    }

    fn has_folder(&self, folder: &NodeRef<Folder>) -> bool {
        self.folders.iter().any(|f| f.ptr_eq(folder))
    }

    fn has_file(&self, file: &NodeRef<File>) -> bool {
        self.files.iter().any(|f| f.ptr_eq(file))
    }

    fn folder_has_name(
        &self,
        folder: &NodeRef<Folder>,
        name: &PathBuf,
    ) -> Result<bool, error::Error> {
        let Some(folder) = folder.upgrade() else {
            return Err(error::Error::InvalidResource);
        };

        let folder = folder.borrow();
        if folder.children.iter().any(|child| {
            let child = child.borrow();
            &child.name == name
        }) {
            return Ok(true);
        }

        if folder.files.iter().any(|file| {
            let file = file.borrow();
            &file.name == name
        }) {
            return Ok(true);
        }

        Ok(false)
    }
}

mod error {
    #[derive(Debug)]
    pub enum Error {
        NotFound,
        NameCollision,
        InvalidResource,
    }
}
