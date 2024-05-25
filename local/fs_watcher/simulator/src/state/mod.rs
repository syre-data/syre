use std::{
    cell::RefCell,
    ffi::OsString,
    ops::Deref,
    path::PathBuf,
    rc::{Rc, Weak},
};
use syre_local::constants;

pub mod action;
pub mod app;
pub mod fs;
pub mod graph;

pub use action::Action;
use app::{
    HasFsDataResource, HasFsDataResourceRelative, HasFsResource, HasFsResourceRelative, HasPath,
    Manifest,
};

pub struct Ptr<T>(Rc<RefCell<T>>);
impl<T> Ptr<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn downgrade(this: &Self) -> WPtr<T> {
        WPtr(Rc::downgrade(&this.0))
    }

    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Rc::ptr_eq(&this.0, &other.0)
    }

    pub fn as_ptr(this: &Self) -> *const RefCell<T> {
        Rc::as_ptr(&this.0)
    }
}

impl<T> std::fmt::Debug for Ptr<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:?} [{:?}]",
            Self::as_ptr(self),
            self.0.borrow()
        ))
    }
}

pub struct WPtr<T>(Weak<RefCell<T>>);
impl<T> WPtr<T> {
    pub fn upgrade(&self) -> Option<Ptr<T>> {
        self.0.upgrade().map(|value| Ptr(value))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }

    pub fn as_ptr(&self) -> *const RefCell<T> {
        self.0.as_ptr()
    }
}

impl<T> std::fmt::Debug for WPtr<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0.as_ptr()))
    }
}

impl<T> Deref for Ptr<T> {
    type Target = Rc<RefCell<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for WPtr<T> {
    type Target = Weak<RefCell<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Clone for WPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait HasName {
    fn name(&self) -> &std::ffi::OsStr;
    fn set_name(&mut self, name: impl Into<std::ffi::OsString>);
}

pub trait Reducible {
    type Action;
    type Output;
    fn reduce(&mut self, action: &Self::Action) -> Result<Self::Output>;
}

#[derive(Debug)]
pub struct State {
    fs: fs::State,
    app: app::State,
}

impl State {
    pub fn new(
        path: impl Into<PathBuf>,
        user_manifest: impl Into<PathBuf>,
        project_manifest: impl Into<PathBuf>,
    ) -> Self {
        Self {
            fs: fs::State::new(path),
            app: app::State::new(user_manifest, project_manifest),
        }
    }

    pub fn fs(&self) -> &fs::State {
        &self.fs
    }

    pub fn app(&self) -> &app::State {
        &self.app
    }
}

impl State {
    /// # Returns
    /// An app resource if it was created.
    /// For folder resources, the root resource is returned.
    pub fn insert_fs_resource(&self, fs_resource: action::FsResource) -> Option<app::AppResource> {
        match fs_resource {
            action::FsResource::File(file) => self
                .insert_file_resource(&file)
                .map(|resource| resource.into()),
            action::FsResource::Folder(folder) => self
                .insert_folder_resource(&folder)
                .map(|resource| resource.into()),
        }
    }

    pub fn remove_fs_resource(
        &mut self,
        fs_resource: action::FsResource,
    ) -> Option<app::AppResource> {
        match fs_resource {
            action::FsResource::File(file) => self
                .remove_file_resource(&file)
                .map(|resource| resource.into()),
            action::FsResource::Folder(folder) => self
                .remove_folder_resource(&folder)
                .map(|resource| resource.into()),
        }
    }

    pub fn insert_file_resource(&self, file: &Ptr<fs::File>) -> Option<app::FileResource> {
        self.app.projects().iter().find_map(|project| {
            let path = self.fs.file_path(file).unwrap();
            if path.starts_with(project.borrow().path()) {
                self.insert_file_project_resource(file, project)
            } else {
                None
            }
        })
    }

    pub fn insert_folder_resource(&self, folder: &Ptr<fs::Folder>) -> Option<app::FolderResource> {
        self.app.projects().iter().find_map(|project| {
            let path = self.fs.graph().path(folder).unwrap();
            if path.starts_with(project.borrow().path()) {
                self.insert_folder_project_resource(folder, project)
            } else {
                None
            }
        })
    }

    fn insert_file_project_resource(
        &self,
        file: &Ptr<fs::File>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FileResource> {
        let path = self.fs.file_path(file).unwrap();
        let project = project.borrow();
        if let app::Resource::Present(config) = project.config() {
            if let Some(analyses) = project.analyses() {
                let analyses = analyses.borrow();
                if let Ok(rel_path) = path.strip_prefix(project.path().join(analyses.path())) {
                    if let Some(ext) = rel_path.extension() {
                        let ext = ext.to_str().unwrap();
                        if syre_core::project::ScriptLang::supported_extensions().contains(&ext) {
                            let config = config.borrow();
                            let mut manifest = config.analyses().borrow_mut();
                            assert!(!manifest
                                .manifest()
                                .iter()
                                .any(|analysis| { analysis.borrow().path() == rel_path }));

                            let analysis = app::Analysis::new(rel_path);
                            let analysis = Ptr::new(analysis);
                            manifest.push(analysis.clone());
                            return Some(app::FileResource::Analysis(analysis));
                        }
                    }
                }
            }
        }

        let data = project.data().borrow();
        if let Ok(rel_path) = path.strip_prefix(project.path().join(data.path())) {
            let graph = data.graph().unwrap();
            let container = graph.find_by_path(rel_path.parent().unwrap()).unwrap();
            let container = container.borrow();
            if let Some(container_data) = container.data() {
                let config = container_data.config().borrow();
                let mut assets = config.assets().borrow_mut();
                let asset = app::Asset::new(rel_path.file_name().unwrap());
                let asset = Ptr::new(asset);
                assets.push(asset.clone());
                return Some(app::FileResource::Asset(asset));
            }
        }

        return None;
    }

    pub fn insert_folder_project_resource(
        &self,
        folder: &Ptr<fs::Folder>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FolderResource> {
        let path = self.fs.graph().path(folder).unwrap();
        let project_ptr = project;
        let project = project_ptr.borrow();
        if path == *project.path() {
            let app_resource = app::FolderResource::Project(project_ptr.clone());
            for child in self.fs.graph().children(folder).unwrap() {
                self.insert_folder_project_resource(&child, project_ptr);
            }

            return Some(app_resource);
        }

        let Ok(rel_path) = path.strip_prefix(project.path()) else {
            return None;
        };

        if let Some(analyses_ptr) = project.analyses() {
            let analyses = analyses_ptr.borrow();
            if rel_path.starts_with(analyses.path()) {
                for child in self.fs.graph().children(folder).unwrap() {
                    self.insert_folder_project_resource(&child, project_ptr);
                }

                for file in folder.borrow().files() {
                    self.insert_file_project_resource(&file, project_ptr);
                }

                return None;
            }
        }

        let data = project.data().clone();
        let mut data = data.borrow_mut();
        if rel_path == data.path() {
            data.create_graph();
            let graph = data.graph().unwrap();
            let container = graph.root();
            for child in self.fs.graph().children(folder).unwrap() {
                self.insert_folder_project_resource(&child, project_ptr);
            }

            for file in folder.borrow().files() {
                self.insert_file_project_resource(file, project_ptr);
            }

            return Some(app::FolderResource::Container(container));
        }

        if rel_path.starts_with(data.path()) {
            let graph = data.graph_mut().unwrap();
            let container = app::Container::without_data(folder.borrow().name());
            let parent = graph.find_by_path(rel_path.parent().unwrap()).unwrap();
            let container = graph.insert(container, &parent).unwrap();
            for child in self.fs.graph().children(folder).unwrap() {
                self.insert_folder_project_resource(&child, project_ptr);
            }

            for file in folder.borrow().files() {
                self.insert_file_project_resource(file, project_ptr);
            }

            return Some(app::FolderResource::Container(container));
        }

        return None;
    }

    pub fn remove_file_resource(&self, file: &Ptr<fs::File>) -> Option<app::FileResource> {
        self.app.projects().iter().find_map(|project| {
            let path = self.fs.file_path(file).unwrap();
            if path.starts_with(project.borrow().path()) {
                self.remove_file_project_resource(file, project)
            } else {
                None
            }
        })
    }

    pub fn remove_file_project_resource(
        &self,
        file: &Ptr<fs::File>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FileResource> {
        let path = self.fs.file_path(file).unwrap();
        let project = project.borrow();
        if let app::Resource::Present(config) = project.config() {
            if let Some(analyses) = project.analyses() {
                let analyses = analyses.borrow();
                if let Ok(rel_path) = path.strip_prefix(project.path().join(analyses.path())) {
                    let config = config.borrow();
                    let mut manifest = config.analyses().borrow_mut();
                    if let Some(index) = manifest
                        .manifest()
                        .iter()
                        .position(|analysis| analysis.borrow().path() == rel_path)
                    {
                        let analysis = manifest.remove(index);
                        return Some(app::FileResource::Analysis(analysis));
                    } else {
                        return None;
                    };
                }
            }
        }

        let data = project.data().borrow();
        if let Ok(rel_path) = path.strip_prefix(project.path().join(data.path())) {
            let graph = data.graph().unwrap();
            let container = graph.find_by_path(rel_path.parent().unwrap()).unwrap();
            let container = container.borrow();
            if let Some(container_data) = container.data() {
                let config = container_data.config().borrow();
                let mut assets = config.assets().borrow_mut();
                let index = assets
                    .manifest()
                    .iter()
                    .position(|asset| asset.borrow().name() == path.file_name().unwrap())
                    .unwrap();

                let asset = assets.remove(index);
                return Some(app::FileResource::Asset(asset));
            }
        }

        return None;
    }

    pub fn remove_folder_resource(
        &mut self,
        folder: &Ptr<fs::Folder>,
    ) -> Option<app::FolderResource> {
        let path = self.fs.graph().path(folder).unwrap();
        let projects = self.app.projects_mut();
        if let Some(index) = projects
            .iter()
            .position(|project| path == *project.borrow().path())
        {
            let project = projects.swap_remove(index);
            return Some(app::FolderResource::Project(project));
        }

        self.app.projects().iter().find_map(|project| {
            if path.starts_with(project.borrow().path()) {
                self.insert_folder_project_resource(folder, project)
            } else {
                None
            }
        })
    }

    fn remove_folder_project_resource(
        &self,
        folder: &Ptr<fs::Folder>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FolderResource> {
        let path = self.fs.graph().path(folder).unwrap();
        let project_ptr = project;
        let project = project_ptr.borrow();
        assert_ne!(path, *project.path());
        let Ok(rel_path) = path.strip_prefix(project.path()) else {
            return None;
        };

        if let Some(analyses_ptr) = project.analyses() {
            let analyses = analyses_ptr.borrow();
            if rel_path.starts_with(analyses.path()) {
                for child in self.fs.graph().children(folder).unwrap() {
                    self.remove_folder_project_resource(&child, project_ptr);
                }

                for file in folder.borrow().files() {
                    self.remove_file_project_resource(&file, project_ptr);
                }

                return None;
            }
        }

        let data = project.data().clone();
        let mut data = data.borrow_mut();
        if rel_path == data.path() {
            let graph = data.graph().unwrap();
            let container = graph.root();
            data.remove_graph();
            return Some(app::FolderResource::Container(container));
        }

        if let Ok(rel_path) = rel_path.strip_prefix(data.path()) {
            let graph = data.graph_mut().unwrap();
            let container = graph.find_by_path(rel_path).unwrap();
            let tree = graph.remove(&container).unwrap();
            return Some(app::FolderResource::ContainerTree(tree));
        }

        return None;
    }
}

impl Reducible for State {
    type Action = Action;
    type Output = ();

    fn reduce(&mut self, action: &Self::Action) -> Result<Self::Output> {
        let fs_resource = self.fs.reduce(action)?;
        match action {
            Action::CreateFolder { path, with_parents } => {
                let action::FsResource::Folder(fs_resource) = fs_resource else {
                    unreachable!();
                };

                self.reduce_create_folder(&fs_resource, path, *with_parents)
            }

            Action::CreateFile { path, with_parents } => {
                let action::FsResource::File(fs_resource) = fs_resource else {
                    unreachable!();
                };

                self.reduce_create_file(&fs_resource, path, *with_parents)
            }

            Action::Remove(path) => {
                match fs_resource {
                    action::FsResource::File(file) => self.reduce_remove_file(&file),
                    action::FsResource::Folder(folder) => self.reduce_remove_folder(&folder),
                }

                Ok(())
            }

            Action::Rename { from, to } => {
                match fs_resource {
                    action::FsResource::File(file) => self.reduce_rename_file(&file, to.clone()),
                    action::FsResource::Folder(folder) => {
                        self.reduce_rename_folder(&folder, to.clone())
                    }
                }

                Ok(())
            }

            Action::Move { from, to } => match fs_resource {
                action::FsResource::File(file) => self.reduce_move_file(from, to, &file),
                action::FsResource::Folder(folder) => self.reduce_move_folder(from, to, &folder),
            },

            Action::Copy { from, to } => match fs_resource {
                action::FsResource::File(file) => self.reduce_copy_file(&file, from, to),
                action::FsResource::Folder(folder) => self.reduce_copy_folder(&folder, from, to),
            },

            Action::Modify { file: path, kind } => {
                let action::FsResource::File(file) = fs_resource else {
                    unreachable!();
                };

                self.reduce_modify(&file, kind);
                Ok(())
            }
        }
    }
}

impl State {
    /// Handle folder creation.
    /// Assumes the folder is new.
    ///
    /// # Panics
    /// + If the folder contains any children or files.
    fn reduce_create_folder(
        &mut self,
        folder: &Ptr<fs::Folder>,
        path: &PathBuf,
        with_parents: bool,
    ) -> Result<<Self as Reducible>::Output> {
        assert!(folder.borrow().files().is_empty());
        assert!(self
            .fs
            .graph()
            .children(folder)
            .expect("children not found")
            .is_empty());

        self.insert_folder_resource(folder);
        Ok(())
    }

    fn reduce_create_file(
        &mut self,
        file: &Ptr<fs::File>,
        path: &PathBuf,
        with_parents: bool,
    ) -> Result<<Self as Reducible>::Output> {
        self.insert_file_resource(file);
        Ok(())
    }

    fn reduce_remove_file(&mut self, file: &Ptr<fs::File>) {
        self.remove_file_resource(file);
    }

    fn reduce_remove_folder(&mut self, folder: &Ptr<fs::Folder>) {
        self.remove_folder_resource(folder);
    }

    fn reduce_rename_file(&mut self, file: &Ptr<fs::File>, to: OsString) {}

    fn reduce_rename_folder(&mut self, folder: &Ptr<fs::Folder>, to: OsString) {}

    fn reduce_move_file(
        &mut self,
        from: &PathBuf,
        to: &PathBuf,
        file: &Ptr<fs::File>,
    ) -> Result<<Self as Reducible>::Output> {
        assert_eq!(self.fs.file_path(file).unwrap(), *to);

        let Some(app_resource) = self.insert_file_resource(file) else {
            return Ok(());
        };

        // TODO: Read file to get validity.
        Ok(())
    }

    fn reduce_move_folder(
        &mut self,
        from: &PathBuf,
        to: &PathBuf,
        folder: &Ptr<fs::Folder>,
    ) -> Result<<Self as Reducible>::Output> {
        Ok(())
    }

    fn reduce_copy_file(
        &mut self,
        file: &Ptr<fs::File>,
        from: &PathBuf,
        to: &PathBuf,
    ) -> Result<<Self as Reducible>::Output> {
        let Some(app_resource) = self.insert_file_resource(file) else {
            return Ok(());
        };

        // TODO: Read file to determine validity.
        Ok(())
    }

    fn reduce_copy_folder(
        &mut self,
        folder: &Ptr<fs::Folder>,
        from: &PathBuf,
        to: &PathBuf,
    ) -> Result<<Self as Reducible>::Output> {
        let Some(app_resource) = self.insert_folder_resource(folder) else {
            return Ok(());
        };

        // TODO: Read contents to determine validity.
        Ok(())
    }

    fn reduce_modify(&mut self, file: &Ptr<fs::File>, kind: &action::ModifyKind) {
        use action::ModifyKind;

        match kind {
            ModifyKind::ManifestAdd(item) => match file.borrow().app_resource().unwrap() {
                app::FileResource::UserManifest(manifest) => {
                    let mut manifest = manifest.borrow_mut();
                    manifest.push(item.into());
                }
                app::FileResource::ProjectManifest(manifest) => {
                    let mut manifest = manifest.borrow_mut();
                    let path: PathBuf = item.into();
                    manifest.push(path.clone());

                    let projects = self.app.projects_mut();
                    if !projects.iter().any(|prj| prj.borrow().path() == &path) {
                        let project = app::Project::new(path.clone(), "data");
                        let project = Ptr::new(project);
                        projects.push(project);
                    }
                }
                app::FileResource::AnalysisManifest(manifest) => {
                    let project = self
                        .app
                        .find_resource_project(
                            app::FileResource::AnalysisManifest(manifest.clone()).into(),
                        )
                        .unwrap();

                    let project = project.borrow();
                    let analyses = project.analyses().unwrap();
                    let analyses = analyses.borrow();
                    let analyses_path = analyses.path();
                    let path = project.path().join(analyses_path).join(item);
                    if let Some(ext) = path.extension() {
                        if syre_core::project::ScriptLang::supported_extensions()
                            .contains(&ext.to_str().unwrap())
                        {
                            let analysis = app::Analysis::new(path.clone());
                            let analysis = Ptr::new(analysis);
                            if let Some(file) = self.fs.find_file(&path) {
                                analysis.borrow_mut().set_fs_resource(&file);
                                file.borrow_mut()
                                    .set_app_resource(app::FileResource::Analysis(Ptr::downgrade(
                                        &analysis,
                                    )));
                            }

                            let manifest = manifest.upgrade().unwrap();
                            manifest.borrow_mut().push(analysis);
                        }
                    }
                }
                app::FileResource::AssetManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    let app::FsDataResource::Present { resource, state: _ } =
                        manifest.fs_resource()
                    else {
                        unreachable!();
                    };

                    let resource = resource.upgrade().unwrap();
                    let config = self.fs.find_file_folder_by_ptr(&resource).unwrap();
                    let container = self.fs.graph().parent(&config).unwrap();
                    let container_path = self.fs.graph().path(&container).unwrap();
                    let path = container_path.join(item);
                    let asset = app::Asset::new(item);
                    let asset = Ptr::new(asset);
                    if let Some(file) = self.fs.find_file(&path) {
                        file.borrow_mut()
                            .set_app_resource(app::FileResource::Asset(Ptr::downgrade(&asset)));
                        asset.borrow_mut().set_fs_resource(&file);
                    }

                    manifest.push(asset);
                }
                app::FileResource::ProjectProperties(_)
                | app::FileResource::ProjectSettings(_)
                | app::FileResource::ContainerProperties(_)
                | app::FileResource::ContainerSettings(_)
                | app::FileResource::Analysis(_)
                | app::FileResource::Asset(_) => unreachable!(),
            },
            ModifyKind::ManifestRemove(index) => match file.borrow().app_resource().unwrap() {
                app::FileResource::UserManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove(*index);
                }
                app::FileResource::ProjectManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove(*index);
                }
                app::FileResource::AnalysisManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove(*index);
                }
                app::FileResource::AssetManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove(*index);
                }
                app::FileResource::ProjectProperties(_)
                | app::FileResource::ProjectSettings(_)
                | app::FileResource::ContainerProperties(_)
                | app::FileResource::ContainerSettings(_)
                | app::FileResource::Analysis(_)
                | app::FileResource::Asset(_) => unreachable!(),
            },
            action::ModifyKind::Corrupt => match file.borrow().app_resource().unwrap() {
                app::FileResource::UserManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::ProjectManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::ProjectProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    let mut properties = properties.borrow_mut();
                    properties.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::ProjectSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    let mut settings = settings.borrow_mut();
                    settings.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::AnalysisManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::ContainerProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    let mut properties = properties.borrow_mut();
                    properties.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::ContainerSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    let mut settings = settings.borrow_mut();
                    settings.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::AssetManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Invalid)
                }
                app::FileResource::Analysis(_) | app::FileResource::Asset(_) => unreachable!(),
            },
            action::ModifyKind::Repair => match file.borrow().app_resource().unwrap() {
                app::FileResource::UserManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::ProjectManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::ProjectProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    let mut properties = properties.borrow_mut();
                    properties.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::ProjectSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    let mut settings = settings.borrow_mut();
                    settings.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::AnalysisManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::ContainerProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    let mut properties = properties.borrow_mut();
                    properties.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::ContainerSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    let mut settings = settings.borrow_mut();
                    settings.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::AssetManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.set_fs_resource(&file, app::DataResourceState::Valid)
                }
                app::FileResource::Analysis(_) | app::FileResource::Asset(_) => unreachable!(),
            },

            action::ModifyKind::Other => {}
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        let (fs_state, fs_node_map) = self.fs.duplicate_with_app_references_and_map();
        let (mut app_state, _app_resource_map) = self.app.duplicate_with_fs_references_and_map();
        let mut user_manifest = app_state.app_state().user_manifest().borrow_mut();
        if let app::FsDataResource::Present { resource, state } =
            user_manifest.fs_resource().clone()
        {
            let resource_ptr = resource.upgrade().unwrap();
            let from_folder = self.fs.find_file_folder_by_ptr(&resource_ptr).unwrap();
            let to_folder = find_mapped_to(&from_folder, &fs_node_map).unwrap();

            let to_resource = to_folder
                .borrow()
                .file(resource_ptr.borrow().name())
                .unwrap();

            user_manifest.set_fs_resource(&to_resource, state);
            to_resource
                .borrow_mut()
                .set_app_resource(app::FileResource::UserManifest(Ptr::downgrade(
                    app_state.app_state().user_manifest(),
                )));
        }
        drop(user_manifest);

        let mut project_manifest = app_state.app_state().project_manifest().borrow_mut();
        if let app::FsDataResource::Present { resource, state } =
            project_manifest.fs_resource().clone()
        {
            let resource_ptr = resource.upgrade().unwrap();
            let from_folder = self.fs.find_file_folder_by_ptr(&resource_ptr).unwrap();
            let to_folder = find_mapped_to(from_folder, &fs_node_map).unwrap();

            let to_resource = to_folder
                .borrow()
                .file(resource_ptr.borrow().name())
                .unwrap();

            project_manifest.set_fs_resource(&to_resource, state);
            to_resource
                .borrow_mut()
                .set_app_resource(app::FileResource::ProjectManifest(Ptr::downgrade(
                    app_state.app_state().project_manifest(),
                )));
        }
        drop(project_manifest);

        for project in app_state.projects().iter() {
            Self::handle_clone_project(project, &fs_node_map);
        }

        Self {
            fs: fs_state,
            app: app_state,
        }
    }
}

impl State {
    fn handle_clone_project(
        project_ptr: &Ptr<app::Project>,
        fs_node_map: &graph::NodeMap<fs::Folder>,
    ) {
        let mut project = project_ptr.borrow_mut();
        if let app::FsResource::Present(resource) = project.fs_resource().clone() {
            let resource_ptr = resource.upgrade().unwrap();
            let project_folder = find_mapped_to(&resource_ptr, &fs_node_map).unwrap();

            project.set_fs_resource(project_folder);
            project_folder
                .borrow_mut()
                .set_app_resource(app::FolderResource::Project(Ptr::downgrade(project_ptr)));
        }

        if let app::Resource::Present(config_ptr) = project.config() {
            let mut config = config_ptr.borrow_mut();
            let resource_ptr = config.fs_resource().upgrade().unwrap();
            let config_folder = find_mapped_to(&resource_ptr, &fs_node_map).unwrap();

            let mut properties = config.properties().borrow_mut();
            if let app::FsDataResource::Present { resource, state } = properties.fs_resource() {
                let file_ptr = resource.upgrade().unwrap();
                let file = config_folder
                    .borrow()
                    .file(file_ptr.borrow().name())
                    .unwrap();

                assert_eq!(file.borrow().name(), constants::PROJECT_FILE);
                *properties = match state {
                    app::DataResourceState::Valid => app::ProjectProperties::valid(&file),
                    app::DataResourceState::Invalid => app::ProjectProperties::invalid(&file),
                };

                file.borrow_mut()
                    .set_app_resource(app::FileResource::ProjectProperties(Ptr::downgrade(
                        config.properties(),
                    )));
            }
            drop(properties);

            let mut settings = config.settings().borrow_mut();
            if let app::FsDataResource::Present { resource, state } = settings.fs_resource() {
                let file_ptr = resource.upgrade().unwrap();
                let file = config_folder
                    .borrow()
                    .file(file_ptr.borrow().name())
                    .unwrap();

                assert_eq!(file.borrow().name(), constants::PROJECT_SETTINGS_FILE);
                *settings = match state {
                    app::DataResourceState::Valid => app::ProjectSettings::valid(&file),
                    app::DataResourceState::Invalid => app::ProjectSettings::invalid(&file),
                };

                file.borrow_mut()
                    .set_app_resource(app::FileResource::ProjectSettings(Ptr::downgrade(
                        config.settings(),
                    )));
            }
            drop(settings);

            let mut analyses = config.analyses().borrow_mut();
            if let app::FsDataResource::Present { resource, state } = analyses.fs_resource().clone()
            {
                let file_ptr = resource.upgrade().unwrap();
                let file = config_folder
                    .borrow()
                    .file(file_ptr.borrow().name())
                    .unwrap();

                assert_eq!(file.borrow().name(), constants::ANALYSES_FILE);
                analyses.set_fs_resource(&file, state);
                file.borrow_mut()
                    .set_app_resource(app::FileResource::AnalysisManifest(Ptr::downgrade(
                        config.analyses(),
                    )));
            }
            drop(analyses);

            config.set_fs_resource(config_folder);
            config_folder
                .borrow_mut()
                .set_app_resource(app::FolderResource::ProjectConfig(Ptr::downgrade(
                    config_ptr,
                )));
        }

        if let Some(analyses) = project.analyses() {
            if let app::FsResource::Present(resource) = analyses.borrow().fs_resource() {
                let folder_ptr = resource.upgrade().unwrap();
                let folder = find_mapped_to(&folder_ptr, &fs_node_map).unwrap();
                analyses.borrow_mut().set_fs_resource(folder)
            }
        }

        let data = project.data().borrow();
        if let Some(graph) = data.graph() {
            Self::handle_clone_project_graph(graph, &fs_node_map);
        }
    }

    fn handle_clone_project_graph(
        graph: &graph::Tree<app::Container>,
        fs_node_map: &graph::NodeMap<fs::Folder>,
    ) {
        for container_ptr in graph.nodes() {
            let mut container = container_ptr.borrow_mut();
            let folder_ptr = container.fs_resource().upgrade().unwrap();
            let container_folder = find_mapped_to(&folder_ptr, &fs_node_map).unwrap();

            if let Some(data) = container.data() {
                let config_ptr = data.config();
                let mut config = config_ptr.borrow_mut();
                let folder_ptr = config.fs_resource().upgrade().unwrap();
                let config_folder = find_mapped_to(&folder_ptr, &fs_node_map).unwrap();
                assert_eq!(config_folder.borrow().name(), constants::APP_DIR);

                if let app::FsDataResource::Present { resource, state } =
                    config.properties().borrow().fs_resource().clone()
                {
                    let file_ptr = resource.upgrade().clone().unwrap();
                    let file = folder_ptr.borrow().file(file_ptr.borrow().name()).unwrap();
                    assert_eq!(file.borrow().name(), constants::CONTAINER_FILE);

                    config
                        .properties()
                        .borrow_mut()
                        .set_fs_resource(&file, state);

                    file.borrow_mut()
                        .set_app_resource(app::FileResource::ContainerProperties(Ptr::downgrade(
                            config.properties(),
                        )));
                }

                if let app::FsDataResource::Present { resource, state } =
                    config.settings().borrow().fs_resource().clone()
                {
                    let file_ptr = resource.upgrade().clone().unwrap();
                    let file = folder_ptr.borrow().file(file_ptr.borrow().name()).unwrap();
                    assert_eq!(file.borrow().name(), constants::CONTAINER_SETTINGS_FILE);

                    config.settings().borrow_mut().set_fs_resource(&file, state);

                    file.borrow_mut()
                        .set_app_resource(app::FileResource::ContainerSettings(Ptr::downgrade(
                            config.settings(),
                        )));
                }

                let mut assets = config.assets().borrow_mut();
                if let app::FsDataResource::Present { resource, state } =
                    assets.fs_resource().clone()
                {
                    let file_ptr = resource.upgrade().clone().unwrap();
                    let file = folder_ptr.borrow().file(file_ptr.borrow().name()).unwrap();
                    assert_eq!(file.borrow().name(), constants::ASSETS_FILE);

                    assets.set_fs_resource(&file, state);
                    file.borrow_mut()
                        .set_app_resource(app::FileResource::AssetManifest(Ptr::downgrade(
                            config.assets(),
                        )));
                }
                drop(assets);

                config.set_fs_resource(config_folder);
                config_folder
                    .borrow_mut()
                    .set_app_resource(app::FolderResource::ContainerConfig(Ptr::downgrade(
                        config_ptr,
                    )));
            }

            container.set_fs_resource(container_folder);
            container_folder
                .borrow_mut()
                .set_app_resource(app::FolderResource::Container(Ptr::downgrade(
                    container_ptr,
                )));
        }
    }
}

/// Finds `origin` in the map, and returns its image.
fn find_mapped_to<'a, T>(origin: &Ptr<T>, map: &'a graph::NodeMap<T>) -> Option<&'a Ptr<T>> {
    map.iter().find_map(|(from, to)| {
        if Ptr::ptr_eq(from, origin) {
            Some(to)
        } else {
            None
        }
    })
}

pub type Result<T = ()> = std::result::Result<T, error::Error>;
pub mod error {
    #[derive(Debug)]
    pub enum Error {
        NameCollision,
        DoesNotExist,
    }
}
