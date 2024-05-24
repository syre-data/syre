use std::{
    cell::RefCell,
    ffi::OsString,
    ops::Deref,
    path::PathBuf,
    rc::{Rc, Weak},
};
use syre_local::{common, constants};

pub mod action;
pub mod app;
pub mod fs;
pub mod graph;

pub use action::Action;
use app::{HasFsDataResource, HasPath, Manifest};

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
    /// Inserts a file system resource into the app state.
    /// If the file system resource corresponds to an app resource,
    /// it is inserted, modifying the state by connecting the file system resource
    /// for that app resource.
    /// Both the file system resource for the app resource, and the app resource for the file
    /// system resource are set.
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

    pub fn insert_file_resource(&self, file: &Ptr<fs::File>) -> Option<app::FileResource> {
        let file_path = self.fs.file_path(file).unwrap();
        let user_manifest_ptr = self.app.app_state().user_manifest();
        let mut user_manifest = user_manifest_ptr.borrow_mut();
        if file_path == *user_manifest.path() {
            assert!(!user_manifest.fs_resource().is_present());
            user_manifest.set_fs_resource(file, app::DataResourceState::Valid);
            file.borrow_mut()
                .set_app_resource(app::FileResource::UserManifest(Ptr::downgrade(
                    user_manifest_ptr,
                )));

            return Some(app::FileResource::UserManifest(Ptr::downgrade(
                user_manifest_ptr,
            )));
        }

        let project_manifest_ptr = self.app.app_state().project_manifest();
        let mut project_manifest = project_manifest_ptr.borrow_mut();
        if file_path == *project_manifest.path() {
            assert!(!project_manifest.fs_resource().is_present());
            project_manifest.set_fs_resource(file, app::DataResourceState::Valid);
            file.borrow_mut()
                .set_app_resource(app::FileResource::ProjectManifest(Ptr::downgrade(
                    project_manifest_ptr,
                )));

            return Some(app::FileResource::ProjectManifest(Ptr::downgrade(
                project_manifest_ptr,
            )));
        }

        self.app
            .projects()
            .iter()
            .find_map(|project| self.insert_file_project_resource(file, project))
    }

    pub fn insert_folder_resource(&self, folder: &Ptr<fs::Folder>) -> Option<app::FolderResource> {
        self.app
            .projects()
            .iter()
            .find_map(|project| self.insert_folder_project_resource(folder, project))
    }

    pub fn insert_file_project_resource(
        &self,
        file: &Ptr<fs::File>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FileResource> {
        let path = self.fs.file_path(file).unwrap();
        let parent = self.fs.find_file_folder_by_ptr(file).unwrap();
        if let Some(app_resource) = parent.borrow().app_resource() {
            match app_resource {
                app::FolderResource::Project(_) => return None,
                app::FolderResource::ProjectConfig(config) => {
                    let config = config.upgrade().unwrap();
                    let config = config.borrow();
                    let filename = path.file_name().unwrap();
                    if filename == constants::PROJECT_FILE {
                        let app_resource = app::FileResource::ProjectProperties(Ptr::downgrade(
                            config.properties(),
                        ));

                        config
                            .properties()
                            .borrow_mut()
                            .set_fs_resource(file, app::DataResourceState::Valid);

                        file.borrow_mut().set_app_resource(app_resource.clone());
                        return Some(app_resource);
                    } else if filename == constants::PROJECT_SETTINGS_FILE {
                        let app_resource =
                            app::FileResource::ProjectSettings(Ptr::downgrade(config.settings()));

                        config
                            .settings()
                            .borrow_mut()
                            .set_fs_resource(file, app::DataResourceState::Valid);

                        file.borrow_mut().set_app_resource(app_resource.clone());
                        return Some(app_resource);
                    } else if filename == constants::ANALYSES_FILE {
                        let app_resource =
                            app::FileResource::AnalysisManifest(Ptr::downgrade(config.analyses()));

                        config
                            .analyses()
                            .borrow_mut()
                            .set_fs_resource(file, app::DataResourceState::Valid);

                        file.borrow_mut().set_app_resource(app_resource.clone());
                        return Some(app_resource);
                    }

                    return None;
                }
                app::FolderResource::Analyses(analyses) => {
                    if let Some(ext) = path.extension() {
                        let ext = ext.to_str().unwrap();
                        if syre_core::project::ScriptLang::supported_extensions().contains(&ext) {
                            let analyses = analyses.upgrade().unwrap();
                            let base_path = project.borrow().path().join(analyses.borrow().path());
                            let rel_path = path.strip_prefix(base_path).unwrap();

                            let mut analysis = app::Analysis::new(rel_path);
                            analysis.set_fs_resource(file);
                            let analysis = Ptr::new(analysis);
                            if let app::Resource::Present(config) = project.borrow().config() {
                                let config = config.borrow();
                                let manifest = config.analyses();
                                manifest.borrow_mut().push(analysis.clone());
                            }

                            let app_resource =
                                app::FileResource::Analysis(Ptr::downgrade(&analysis));

                            file.borrow_mut().set_app_resource(app_resource.clone());
                            return Some(app_resource);
                        }
                    }

                    return None;
                }
                app::FolderResource::Container(container) => {
                    let container = container.upgrade().unwrap();
                    let mut asset = app::Asset::new(path.file_name().unwrap());
                    asset.set_fs_resource(file);
                    let asset = Ptr::new(asset);
                    let app_resource = app::FileResource::Asset(Ptr::downgrade(&asset));
                    file.borrow_mut().set_app_resource(app_resource.clone());

                    if let Some(data) = container.borrow().data() {
                        let config = data.config().borrow();
                        config.assets().borrow_mut().push(asset.clone());
                    }

                    return Some(app_resource);
                }
                app::FolderResource::ContainerConfig(config) => {
                    let config = config.upgrade().unwrap();
                    let config = config.borrow();
                    let filename = path.file_name().unwrap();
                    if filename == constants::CONTAINER_FILE {
                        let app_resource = app::FileResource::ContainerProperties(Ptr::downgrade(
                            config.properties(),
                        ));

                        config
                            .properties()
                            .borrow_mut()
                            .set_fs_resource(file, app::DataResourceState::Valid);

                        file.borrow_mut().set_app_resource(app_resource.clone());
                        return Some(app_resource);
                    } else if filename == constants::CONTAINER_SETTINGS_FILE {
                        let app_resource =
                            app::FileResource::ContainerSettings(Ptr::downgrade(config.settings()));

                        config
                            .settings()
                            .borrow_mut()
                            .set_fs_resource(file, app::DataResourceState::Valid);

                        file.borrow_mut().set_app_resource(app_resource.clone());
                        return Some(app_resource);
                    } else if filename == constants::ASSETS_FILE {
                        let app_resource =
                            app::FileResource::AssetManifest(Ptr::downgrade(config.assets()));

                        config
                            .assets()
                            .borrow_mut()
                            .set_fs_resource(file, app::DataResourceState::Valid);

                        file.borrow_mut().set_app_resource(app_resource.clone());
                        return Some(app_resource);
                    }

                    return None;
                }
            }
        };

        return None;
    }

    pub fn insert_folder_project_resource(
        &self,
        folder: &Ptr<fs::Folder>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FolderResource> {
        let path = self.fs.graph().path(folder).unwrap();
        let project_ptr = project;
        let mut project = project_ptr.borrow_mut();
        if path == *project.path() {
            assert!(!project.fs_resource().is_present());
            project.set_fs_resource(folder);
            let app_resource = app::FolderResource::Project(Ptr::downgrade(project_ptr));
            folder.borrow_mut().set_app_resource(app_resource.clone());
            for child in self.fs.graph().children(folder).unwrap() {
                self.insert_folder_project_resource(&child, project_ptr);
            }

            return Some(app_resource);
        }

        let Ok(rel_path) = path.strip_prefix(project.path()) else {
            return None;
        };

        if let Some(analyses_ptr) = project.analyses() {
            let mut analyses = analyses_ptr.borrow_mut();
            if rel_path == analyses.path() {
                assert!(!analyses.fs_resource().is_present());
                analyses.set_fs_resource(folder);
                let app_resource = app::FolderResource::Analyses(Ptr::downgrade(analyses_ptr));

                let mut folder = folder.borrow_mut();
                folder.set_app_resource(app_resource.clone());
                for file in folder.files() {
                    self.insert_file_project_resource(&file, project_ptr);
                }

                return Some(app_resource);
            }
        }

        let data = project.data().clone();
        let mut data = data.borrow_mut();
        if rel_path == data.path() {
            assert!(
                data.graph().is_none(),
                "should only be able to create new folder at path if a resource does not already exist there"
            );

            data.set_graph_root(folder);
            let graph = data.graph().as_ref().unwrap();
            let container = graph.root();
            container.borrow_mut().set_fs_resource(folder);

            let app_resource = app::FolderResource::Container(Ptr::downgrade(&container));
            let folder_ptr = folder;
            let mut folder = folder_ptr.borrow_mut();
            folder.set_app_resource(app_resource.clone());

            for child in self.fs.graph().children(folder_ptr).unwrap() {
                self.insert_folder_project_resource(&child, project_ptr);
            }

            for file in folder.files() {
                self.insert_file_project_resource(file, project_ptr);
            }

            return Some(app_resource);
        }

        let parent = self.fs.graph().parent(folder).unwrap();
        let parent = parent.borrow();
        if let Some(parent_resource) = parent.app_resource() {
            match parent_resource {
                app::FolderResource::Project(parent) => {
                    let parent = parent.upgrade().unwrap();
                    assert!(Ptr::ptr_eq(&parent, project_ptr));

                    if rel_path == common::app_dir() {
                        assert!(!project.config().is_present());
                        project.set_config_folder(folder);

                        let app::Resource::Present(config_ptr) = project.config() else {
                            unreachable!();
                        };

                        let mut folder = folder.borrow_mut();
                        let app_resource =
                            app::FolderResource::ProjectConfig(Ptr::downgrade(config_ptr));

                        folder.set_app_resource(app_resource.clone());
                        for file in folder.files().iter() {
                            self.insert_file_resource(file);
                        }

                        return Some(app_resource);
                    }
                }

                app::FolderResource::Container(parent_ptr) => {
                    let parent_ptr = parent_ptr.upgrade().unwrap();
                    let graph = data.graph().as_ref().unwrap();
                    let parent_path = graph.path(&parent_ptr).unwrap();
                    let mut parent = parent_ptr.borrow_mut();
                    let rel_path = rel_path.strip_prefix(parent_path).unwrap();

                    if rel_path == common::app_dir() {
                        assert!(parent.data().is_none());
                        let data = app::ContainerData::new(folder);
                        let app_resource =
                            app::FolderResource::ContainerConfig(Ptr::downgrade(data.config()));

                        parent.set_data(data);
                        let mut folder = folder.borrow_mut();
                        folder.set_app_resource(app_resource.clone());
                        for file in folder.files() {
                            self.insert_file_project_resource(file, project_ptr);
                        }

                        return Some(app_resource);
                    } else {
                        let mut container = app::Container::without_data(folder);
                        container.set_fs_resource(folder);
                        let container_ptr = Ptr::new(container);
                        let app_resource =
                            app::FolderResource::Container(Ptr::downgrade(&container_ptr));

                        let folder_ptr = folder;
                        let mut folder = folder_ptr.borrow_mut();
                        folder.set_app_resource(app_resource.clone());

                        for child in self.fs.graph().children(folder_ptr).unwrap() {
                            self.insert_folder_project_resource(folder_ptr, project_ptr);
                        }

                        for file in folder.files() {
                            self.insert_file_project_resource(file, project_ptr);
                        }

                        return Some(app_resource);
                    }
                }
                app::FolderResource::ProjectConfig(_)
                | app::FolderResource::Analyses(_)
                | app::FolderResource::ContainerConfig(_) => return None,
            }
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
        let mut file = file.borrow_mut();
        if let Some(app_resource) = file.app_resource() {
            match app_resource {
                app::FileResource::UserManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ProjectManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ProjectProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    let mut properties = properties.borrow_mut();
                    properties.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ProjectSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    let mut settings = settings.borrow_mut();
                    settings.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::AnalysisManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::Analysis(analysis) => {
                    let project = self
                        .app
                        .find_resource_project(app::FileResource::Analysis(analysis.clone()).into())
                        .unwrap();

                    let project = project.borrow();
                    let app::Resource::Present(config) = project.config() else {
                        unreachable!();
                    };

                    let config = config.borrow();
                    let mut manifest = config.analyses().borrow_mut();
                    let analysis = analysis.upgrade().unwrap();
                    let index = manifest
                        .manifest()
                        .iter()
                        .position(|a| Ptr::ptr_eq(&analysis, a))
                        .unwrap();

                    manifest.remove(index);
                    file.remove_app_resource();
                }
                app::FileResource::ContainerProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    let mut properties = properties.borrow_mut();
                    properties.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ContainerSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    let mut settings = settings.borrow_mut();
                    settings.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::AssetManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::Asset(asset) => {
                    todo!();
                }
            }
        }
    }

    fn reduce_remove_folder(&mut self, folder_ptr: &Ptr<fs::Folder>) {
        let mut folder = folder_ptr.borrow_mut();
        if let Some(app_resource) = folder.app_resource() {
            match app_resource {
                app::FolderResource::Project(project) => {
                    let project = project.upgrade().unwrap();
                    self.app
                        .projects_mut()
                        .retain(|prj| !Ptr::ptr_eq(&project, prj));

                    // TODO: Not sure if we should also remove from manifest,
                    // or let that be a seperate action.
                    let mut project_manifest = self.app.app_state().project_manifest().borrow_mut();
                    if let Some(index) = project_manifest
                        .manifest()
                        .iter()
                        .position(|path| path == project.borrow().path())
                    {
                        project_manifest.remove(index);
                    }
                }

                app::FolderResource::ProjectConfig(config) => {
                    let project = self
                        .app
                        .find_resource_project(app::FolderResource::ProjectConfig(config).into())
                        .unwrap();

                    project.borrow_mut().remove_config();
                    folder.remove_app_resource();
                }

                app::FolderResource::Analyses(analyses) => {
                    let project = self
                        .app
                        .find_resource_project(app::FolderResource::Analyses(analyses).into())
                        .unwrap();

                    project.borrow_mut().remove_analyses_folder_reference();
                    folder.remove_app_resource();
                }

                app::FolderResource::Container(container) => {
                    let project = self
                        .app
                        .find_resource_project(
                            app::FolderResource::Container(container.clone()).into(),
                        )
                        .unwrap();

                    let project = project.borrow();
                    let mut data = project.data().borrow_mut();
                    let container = container.upgrade().unwrap();
                    data.remove_container(&container);
                    folder.remove_app_resource();
                }

                app::FolderResource::ContainerConfig(_) => todo!(),
            }
        }
    }

    fn reduce_rename_file(&mut self, file: &Ptr<fs::File>, to: OsString) {
        let mut file = file.borrow_mut();
        if let Some(app_resource) = file.app_resource() {
            match app_resource {
                app::FileResource::Asset(asset) => {
                    let asset = asset.upgrade().unwrap();
                    asset.borrow_mut().set_name(to);
                }
                app::FileResource::UserManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    manifest.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ProjectManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    manifest.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ProjectProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    properties.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ProjectSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    settings.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::AnalysisManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    manifest.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::Analysis(analysis) => {
                    fn remove_analysis(
                        analysis: WPtr<app::Analysis>,
                        mut file: std::cell::RefMut<fs::File>,
                        state: &app::State,
                    ) {
                        let project = state
                            .find_resource_project(
                                app::FileResource::Analysis(analysis.clone()).into(),
                            )
                            .unwrap();
                        let project = project.borrow();
                        let app::Resource::Present(config) = project.config() else {
                            unreachable!();
                        };

                        let config = config.borrow();
                        let mut analyses = config.analyses().borrow_mut();
                        let analysis = analysis.upgrade().unwrap();
                        let index = analyses
                            .manifest()
                            .iter()
                            .position(|a| Ptr::ptr_eq(&analysis, a))
                            .unwrap();

                        analyses.remove(index);
                        file.remove_app_resource();
                    }

                    if let Some(ext) = PathBuf::from(file.name()).extension() {
                        if !syre_core::project::ScriptLang::supported_extensions()
                            .contains(&ext.to_str().unwrap())
                        {
                            remove_analysis(analysis, file, &self.app);
                        }
                    } else {
                        remove_analysis(analysis, file, &self.app);
                    }
                }
                app::FileResource::ContainerProperties(properties) => {
                    let properties = properties.upgrade().unwrap();
                    properties.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::ContainerSettings(settings) => {
                    let settings = settings.upgrade().unwrap();
                    settings.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
                app::FileResource::AssetManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    manifest.borrow_mut().remove_fs_resource();
                    file.remove_app_resource();
                }
            }
        }
    }

    fn reduce_rename_folder(&mut self, folder: &Ptr<fs::Folder>, to: OsString) {
        let folder_path = self.fs.graph().path(folder).unwrap();
        let mut folder_resource = folder.borrow_mut();
        if let Some(app_resource) = folder_resource.app_resource() {
            match app_resource {
                app::FolderResource::Project(project) => {
                    let project = project.upgrade().unwrap();
                    let mut project_manifest = self.app.app_state().project_manifest().borrow_mut();
                    if let Some(index) = project_manifest
                        .manifest()
                        .iter()
                        .position(|path| path == project.borrow().path())
                    {
                        project_manifest.remove(index);
                        project_manifest.push(folder_path.clone());
                    }

                    project.borrow_mut().set_path(folder_path);
                }

                app::FolderResource::ProjectConfig(config) => {
                    let project = self
                        .app
                        .find_resource_project(
                            app::FolderResource::ProjectConfig(config.clone()).into(),
                        )
                        .unwrap();

                    let config = config.upgrade().unwrap();
                    let config = config.borrow();
                    if let app::FsDataResource::Present { resource, .. } =
                        config.properties().borrow().fs_resource()
                    {
                        let resource = resource.upgrade().unwrap();
                        resource.borrow_mut().remove_app_resource();
                    };

                    if let app::FsDataResource::Present { resource, .. } =
                        config.settings().borrow().fs_resource()
                    {
                        let resource = resource.upgrade().unwrap();
                        resource.borrow_mut().remove_app_resource();
                    };

                    if let app::FsDataResource::Present { resource, .. } =
                        config.analyses().borrow().fs_resource()
                    {
                        let resource = resource.upgrade().unwrap();
                        resource.borrow_mut().remove_app_resource();
                    };

                    folder_resource.remove_app_resource();
                    project.borrow_mut().remove_config();
                }
                app::FolderResource::Analyses(analyses) => todo!(),
                app::FolderResource::Container(container) => {
                    let project = self
                        .app
                        .find_resource_project(
                            app::FolderResource::Container(container.clone()).into(),
                        )
                        .unwrap();

                    let project = project.borrow();
                    let mut data = project.data().borrow_mut();
                    let graph = data.graph().as_ref().unwrap();
                    let container = container.upgrade().unwrap();
                    if Ptr::ptr_eq(&container, &graph.root()) {
                        let mut path = data.path().clone();
                        path.set_file_name(to.clone());
                        data.set_path(path);
                    }

                    container.borrow_mut().set_name(to);
                }
                app::FolderResource::ContainerConfig(_) => todo!(),
            }
        }
    }

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
        if let Some(app_resource) = folder.borrow().app_resource() {
            if let app::FolderResource::Project(project) = app_resource {
                let project_manifest = self.app.app_state().project_manifest();
                let mut project_manifest = project_manifest.borrow_mut();
                if let Some(index) = project_manifest
                    .manifest()
                    .iter()
                    .position(|path| path == from)
                {
                    project_manifest.remove(index);
                    project_manifest.push(to.clone());
                }

                let project = project.upgrade().unwrap();
                project.borrow_mut().set_path(to.clone());
                return Ok(());
            }
        }

        // TODO: Read file to get validity.
        self.reduce_remove_folder(&folder);
        self.insert_folder_resource(folder);
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
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    manifest.push(item.into());
                }
                app::FileResource::ProjectManifest(manifest) => {
                    let manifest = manifest.upgrade().unwrap();
                    let mut manifest = manifest.borrow_mut();
                    let path: PathBuf = item.into();
                    manifest.push(path.clone());

                    let projects = self.app.projects_mut();
                    if !projects.iter().any(|prj| prj.borrow().path() == &path) {
                        let project = app::Project::new(path.clone(), "data");
                        let project = Ptr::new(project);
                        if let Some(folder) = self.fs.graph().find_by_path(&path) {
                            project.borrow_mut().set_fs_resource(&folder);
                            folder
                                .borrow_mut()
                                .set_app_resource(app::FolderResource::Project(Ptr::downgrade(
                                    &project,
                                )));
                        }
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
