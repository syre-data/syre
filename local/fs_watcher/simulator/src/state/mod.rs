use std::{
    cell::RefCell,
    ffi::OsString,
    ops::Deref,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
};
use syre_local::{common, constants};

pub mod app;
pub mod fs;
pub mod graph;

use app::{HasPath, Manifest};

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
    pub fn insert_fs_resource(&self, fs_resource: fs::Resource) -> Option<app::AppResource> {
        match fs_resource {
            fs::Resource::File(file) => self
                .insert_file_resource(&file)
                .map(|resource| resource.into()),
            fs::Resource::Folder(folder) => self
                .insert_folder_resource(&folder)
                .map(|resource| resource.into()),
        }
    }

    pub fn insert_file_resource(&self, file: &Ptr<fs::File>) -> Option<app::FileResource> {
        let file_path = self.fs.file_path(file).unwrap();
        let user_manifest_ptr = self.app.app_state().user_manifest();
        let user_manifest = user_manifest_ptr.borrow_mut();
        if file_path == *user_manifest.path() {
            return Some(app::FileResource::UserManifest(Ptr::downgrade(
                user_manifest_ptr,
            )));
        }

        let project_manifest_ptr = self.app.app_state().project_manifest();
        let project_manifest = project_manifest_ptr.borrow_mut();
        if file_path == *project_manifest.path() {
            return Some(app::FileResource::ProjectManifest(Ptr::downgrade(
                project_manifest_ptr,
            )));
        }

        self.app
            .projects()
            .iter()
            .find_map(|project| self.insert_file_project_resource(file, project))
    }

    pub fn insert_folder_resource(&self, path: &PathBuf) -> Option<app::FolderResource> {
        self.app.projects().iter().find_map(|project| {
            if let Ok(rel_path) = path.strip_prefix(project.borrow().path()) {
                self.insert_folder_project_resource(rel_path, project)
            } else {
                None
            }
        })
    }

    /// # Arguments
    /// #. `path`: relative path to the file from the project root.
    pub fn insert_file_project_resource(
        &self,
        path: impl AsRef<Path>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FileResource> {
        let path = path.as_ref();
        if let Some(parent_resource) = self
            .app
            .find_path_project_resource(path.parent().unwrap(), project)
        {
            let app::AppResource::Folder(parent_resource) = parent_resource else {
                unreachable!();
            };

            match parent_resource {
                app::FolderResource::Project(_) => return None,
                app::FolderResource::ProjectConfig(config) => {
                    let config = config.upgrade().unwrap();
                    let config = config.borrow();
                    let filename = path.file_name().unwrap();
                    if filename == constants::PROJECT_FILE {
                        let app_resource = app::FileResource::ProjectProperties(Ptr::downgrade(
                            config.properties(),
                        ));

                        return Some(app_resource);
                    } else if filename == constants::PROJECT_SETTINGS_FILE {
                        let app_resource =
                            app::FileResource::ProjectSettings(Ptr::downgrade(config.settings()));

                        return Some(app_resource);
                    } else if filename == constants::ANALYSES_FILE {
                        let app_resource =
                            app::FileResource::AnalysisManifest(Ptr::downgrade(config.analyses()));

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

                            let analysis = app::Analysis::new(rel_path);
                            let analysis = Ptr::new(analysis);
                            if let app::Resource::Present(config) = project.borrow().config() {
                                let config = config.borrow();
                                let manifest = config.analyses();
                                manifest.borrow_mut().push(analysis.clone());
                            }

                            let app_resource =
                                app::FileResource::Analysis(Ptr::downgrade(&analysis));

                            return Some(app_resource);
                        }
                    }

                    return None;
                }
                app::FolderResource::Container(container) => {
                    let container = container.upgrade().unwrap();
                    let asset = app::Asset::new(path.file_name().unwrap());
                    let asset = Ptr::new(asset);
                    let app_resource = app::FileResource::Asset(Ptr::downgrade(&asset));

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

                        return Some(app_resource);
                    } else if filename == constants::CONTAINER_SETTINGS_FILE {
                        let app_resource =
                            app::FileResource::ContainerSettings(Ptr::downgrade(config.settings()));
                        return Some(app_resource);
                    } else if filename == constants::ASSETS_FILE {
                        let app_resource =
                            app::FileResource::AssetManifest(Ptr::downgrade(config.assets()));
                        return Some(app_resource);
                    }

                    return None;
                }
            }
        };

        return None;
    }

    /// # Arguments
    /// #. `path`: Relative path to the folder from the project's root.
    pub fn insert_folder_project_resource(
        &self,
        path: impl AsRef<Path>,
        project: &Ptr<app::Project>,
    ) -> Option<app::FolderResource> {
        let path = path.as_ref();
        let project_ptr = project;
        let mut project = project_ptr.borrow_mut();
        if path == project.path() {
            let folder = self.fs.graph().find_by_path(path).unwrap();
            for child in self.fs.graph().children(&folder).unwrap() {
                let path = self.fs.graph().path(&child).unwrap();
                self.insert_folder_project_resource(&path, project_ptr);
            }

            let app_resource = app::FolderResource::Project(Ptr::downgrade(project_ptr));
            return Some(app_resource);
        }

        let Ok(rel_path) = path.strip_prefix(project.path()) else {
            return None;
        };

        if rel_path == common::app_dir() {
            assert!(!project.config().is_present());
            let folder = self.fs.graph().find_by_path(path).unwrap();
            project.set_config_folder(&folder);

            let app::Resource::Present(config_ptr) = project.config() else {
                unreachable!();
            };

            let folder = folder.borrow();
            let app_resource = app::FolderResource::ProjectConfig(Ptr::downgrade(config_ptr));

            for file in folder.files().iter() {
                self.insert_file_resource(file);
            }

            return Some(app_resource);
        }

        if let Some(analyses_ptr) = project.analyses() {
            let analyses = analyses_ptr.borrow();
            if rel_path == analyses.path() {
                let folder = self.fs.graph().find_by_path(path).unwrap();
                let folder = folder.borrow();
                for file in folder.files() {
                    let path = self.fs.file_path(file).unwrap();
                    let path = path.strip_prefix(project.path()).unwrap();
                    self.insert_file_project_resource(&path, project_ptr);
                }

                let app_resource = app::FolderResource::Analyses(Ptr::downgrade(analyses_ptr));
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

            let folder = self.fs.graph().find_by_path(path).unwrap();
            data.set_graph_root(folder.borrow().name());
            let graph = data.graph().as_ref().unwrap();
            let container = graph.root();

            let app_resource = app::FolderResource::Container(Ptr::downgrade(&container));
            let folder_ptr = folder;
            let folder = folder_ptr.borrow();
            for child in self.fs.graph().children(&folder_ptr).unwrap() {
                let path = self.fs.graph().path(&child).unwrap();
                self.insert_folder_project_resource(&path, project_ptr);
            }

            for file in folder.files() {
                let path = self.fs.file_path(file).unwrap();
                let path = path.strip_prefix(project.path()).unwrap();
                self.insert_file_project_resource(&path, project_ptr);
            }

            return Some(app_resource);
        } else if let Ok(rel_path) = rel_path.strip_prefix(data.path()) {
            if rel_path == common::app_dir() {
                let app::AppResource::Folder(app::FolderResource::Container(parent)) =
                    self.app.find_path_resource(path.parent().unwrap()).unwrap()
                else {
                    panic!();
                };

                let parent = parent.upgrade().unwrap();
                assert!(parent.borrow().data().is_none());
                let data = app::ContainerData::new(folder);
                let app_resource =
                    app::FolderResource::ContainerConfig(Ptr::downgrade(data.config()));

                parent.borrow_mut().set_data(data);
                let folder = folder.borrow();
                for file in folder.files() {
                    self.insert_file_project_resource(file, project_ptr);
                }

                return Some(app_resource);
            } else {
                let container = app::Container::new(folder.borrow().name());
                let container_ptr = Ptr::new(container);
                let app_resource = app::FolderResource::Container(Ptr::downgrade(&container_ptr));

                let folder_ptr = folder;
                let folder = folder_ptr.borrow_mut();

                for child in self.fs.graph().children(folder_ptr).unwrap() {
                    self.insert_folder_project_resource(&child, project_ptr);
                }

                for file in folder.files() {
                    self.insert_file_project_resource(file, project_ptr);
                }

                return Some(app_resource);
            }
        }

        // let parent = self.fs.graph().parent(folder).unwrap();
        // let parent = parent.borrow();
        // if let Some(parent_resource) = self.app.find_resource {
        //     match parent_resource {
        //         app::FolderResource::Container(parent_ptr) => {
        //             let parent_ptr = parent_ptr.upgrade().unwrap();
        //             let graph = data.graph().as_ref().unwrap();
        //             let parent_path = graph.path(&parent_ptr).unwrap();
        //             let mut parent = parent_ptr.borrow_mut();
        //             let rel_path = rel_path.strip_prefix(parent_path).unwrap();
        //         }
        //         app::FolderResource::Project(_)
        //         | app::FolderResource::ProjectConfig(_)
        //         | app::FolderResource::Analyses(_)
        //         | app::FolderResource::ContainerConfig(_) => return None,
        //     }
        // }

        return None;
    }
}

impl Reducible for State {
    type Action = fs::Action;
    type Output = ();

    fn reduce(&mut self, action: &Self::Action) -> Result<Self::Output> {
        let fs_resource = self.fs.reduce(action)?;
        match action {
            fs::Action::CreateFolder { path, with_parents } => {
                let fs::Resource::Folder(fs_resource) = fs_resource else {
                    unreachable!();
                };

                self.reduce_create_folder(&fs_resource, path, *with_parents)
            }

            fs::Action::CreateFile { path, with_parents } => {
                let fs::Resource::File(fs_resource) = fs_resource else {
                    unreachable!();
                };

                self.reduce_create_file(&fs_resource, path, *with_parents)
            }

            fs::Action::Remove(path) => {
                match fs_resource {
                    fs::Resource::File(file) => self.reduce_remove_file(&file),
                    fs::Resource::Folder(folder) => self.reduce_remove_folder(&folder),
                }

                Ok(())
            }

            fs::Action::Rename { from, to } => {
                match fs_resource {
                    fs::Resource::File(file) => self.reduce_rename_file(from, to.clone()),
                    fs::Resource::Folder(folder) => self.reduce_rename_folder(&folder, to.clone()),
                }

                Ok(())
            }

            fs::Action::Move { from, to } => match fs_resource {
                fs::Resource::File(file) => self.reduce_move_file(from, to, &file),
                fs::Resource::Folder(folder) => self.reduce_move_folder(from, to, &folder),
            },

            fs::Action::Copy { from, to } => match fs_resource {
                fs::Resource::File(file) => self.reduce_copy_file(&file, from, to),
                fs::Resource::Folder(folder) => self.reduce_copy_folder(&folder, from, to),
            },

            fs::Action::Modify { file: path, kind } => {
                let fs::Resource::File(file) = fs_resource else {
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
                }
                app::FileResource::Asset(asset) => {
                    todo!();
                }
                app::FileResource::UserManifest(_)
                | app::FileResource::ProjectManifest(_)
                | app::FileResource::ProjectProperties(_)
                | app::FileResource::ProjectSettings(_)
                | app::FileResource::AnalysisManifest(_)
                | app::FileResource::ContainerProperties(_)
                | app::FileResource::ContainerSettings(_)
                | app::FileResource::AssetManifest(_) => {}
            }
        }
    }

    fn reduce_remove_folder(&mut self, path: &PathBuf) {
        if let Some(app_resource) = self.app.find_path_resource(path) {
            let app::AppResource::Folder(app_resource) = app_resource else {
                unreachable!();
            };

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
                }

                app::FolderResource::ContainerConfig(_) => todo!(),
                app::FolderResource::Analyses(analyses) => {}
            }
        }
    }

    fn reduce_rename_file(&mut self, from: &PathBuf, to: OsString) {
        if let Some(app_resource) = self.app.find_path_resource(from) {
            let app::AppResource::File(app_resource) = app_resource else {
                unreachable!();
            };

            match app_resource {
                app::FileResource::Asset(asset) => {
                    let asset = asset.upgrade().unwrap();
                    asset.borrow_mut().set_name(to);
                }
                app::FileResource::Analysis(analysis) => {
                    fn remove_analysis(analysis: WPtr<app::Analysis>, state: &app::State) {
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
                    }

                    if let Some(ext) = PathBuf::from(to).extension() {
                        if !syre_core::project::ScriptLang::supported_extensions()
                            .contains(&ext.to_str().unwrap())
                        {
                            remove_analysis(analysis, &self.app);
                        }
                    } else {
                        remove_analysis(analysis, &self.app);
                    }
                }
                app::FileResource::UserManifest(_)
                | app::FileResource::ProjectManifest(_)
                | app::FileResource::ProjectProperties(_)
                | app::FileResource::ProjectSettings(_)
                | app::FileResource::AnalysisManifest(_)
                | app::FileResource::ContainerProperties(_)
                | app::FileResource::ContainerSettings(_)
                | app::FileResource::AssetManifest(_) => {}
            }
        }
    }

    fn reduce_rename_folder(&mut self, from: &PathBuf, to: OsString) {
        if let Some(app_resource) = self.app.find_path_resource(from) {
            let app::AppResource::Folder(app_resource) = app_resource else {
                unreachable!();
            };

            match app_resource {
                app::FolderResource::Project(project) => {
                    let project = project.upgrade().unwrap();
                    let mut new_path = from.clone();
                    new_path.set_file_name(to.clone());
                    let mut project_manifest = self.app.app_state().project_manifest().borrow_mut();
                    if let Some(index) = project_manifest
                        .manifest()
                        .iter()
                        .position(|path| path == project.borrow().path())
                    {
                        project_manifest.remove(index);
                        project_manifest.push(new_path.clone());
                    }

                    project.borrow_mut().set_path(new_path);
                }

                app::FolderResource::ProjectConfig(config) => {
                    let project = self
                        .app
                        .find_resource_project(
                            app::FolderResource::ProjectConfig(config.clone()).into(),
                        )
                        .unwrap();

                    project.borrow_mut().remove_config();
                }
                app::FolderResource::Analyses(_analyses) => todo!(),
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
    ) -> Result<<Self as Reducible>::Output> {
        if let Some(app_resource) = self.app.find_path_resource(from) {
            let app::AppResource::Folder(app_resource) = app_resource else {
                unreachable!();
            };

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

        self.reduce_remove_folder(from);
        self.insert_folder_resource(to);
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

    fn reduce_modify(&mut self, file: &Ptr<fs::File>, kind: &fs::ModifyKind) {
        use fs::ModifyKind;

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
            fs::ModifyKind::Corrupt => match file.borrow().app_resource().unwrap() {
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
            fs::ModifyKind::Repair => match file.borrow().app_resource().unwrap() {
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

            fs::ModifyKind::Other => {}
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
