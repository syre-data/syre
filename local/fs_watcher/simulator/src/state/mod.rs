use std::{
    assert_matches::assert_matches,
    cell::RefCell,
    ops::Deref,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
};
use syre_local::{common, constants};

pub mod action;
pub mod app;
pub mod fs;
pub mod graph;

pub use action::Action;
use app::{HasFsDataResource, HasPath};

#[derive(Debug)]
pub struct Ptr<T>(Rc<RefCell<T>>);
impl<T> Ptr<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn downgrade(this: &Self) -> WPtr<T> {
        WPtr(Rc::downgrade(&this.0))
    }

    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Rc::ptr_eq(this, other)
    }
}

#[derive(Debug)]
pub struct WPtr<T>(Weak<RefCell<T>>);
impl<T> WPtr<T> {
    pub fn upgrade(&self) -> Option<Ptr<T>> {
        self.0.upgrade().map(|value| Ptr(value))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(other)
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
        path: impl AsRef<Path>,
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

impl Reducible for State {
    type Action = Action;
    type Output = ();

    fn reduce(&mut self, action: &Self::Action) -> Result<Self::Output> {
        let fs_resource = self.fs.reduce(action)?;
        match action {
            Action::CreateFolder { parent, name } => {
                let action::FsResource::Folder(fs_resource) = fs_resource else {
                    unreachable!();
                };

                let Some(app_resource) = parent.borrow().app_resource() else {
                    return Ok(());
                };

                match app_resource {
                    app::FolderResource::Project(project) => {
                        let project = project.upgrade().unwrap();
                        let rel_path = self.fs.graph().path(&fs_resource).unwrap();
                        let rel_path = rel_path.strip_prefix(project.borrow().path()).unwrap();

                        if name == common::app_dir() {
                            assert_matches!(
                                project.borrow().config(),
                                app::Resource::NotPresent,
                                "should only be able to create new folder at path if a resource does not already exist there"
                            );

                            project.borrow_mut().set_config_folder(&fs_resource);
                        } else if let Some(analyses) = project.borrow().analyses() {
                            if rel_path == analyses.borrow().path() {
                                assert_matches!(
                                    analyses.borrow().fs_resource(),
                                    app::FsResource::NotPresent
                                );

                                project
                                    .borrow_mut()
                                    .set_analyses_folder_reference(&fs_resource);
                            }
                        } else if rel_path == project.borrow().data().borrow().path() {
                            assert_matches!(
                                project.borrow().data().borrow().graph(),
                                None,
                                "should only be able to create new folder at path if a resource does not already exist there"
                            );

                            project.borrow_mut().set_data_root(&fs_resource);
                        } else {
                            todo!();
                        }
                    }

                    _ => todo!(),
                }
                Ok(())
            }

            Action::CreateFolderAt { path, with_parents } => todo!(),

            Action::CreateFile { parent, name } => {
                let action::FsResource::File(fs_resource) = fs_resource else {
                    unreachable!();
                };

                todo!();
                Ok(())
            }

            Action::CreateFileAt { path, with_parents } => {
                let action::FsResource::File(fs_resource) = fs_resource else {
                    unreachable!();
                };

                let mut user_manifest = self.app.app_state().user_manifest().borrow_mut();
                let mut project_manifest = self.app.app_state().project_manifest().borrow_mut();
                if path == user_manifest.path() {
                    user_manifest.set_fs_resource(&fs_resource, app::DataResourceState::Valid);
                    fs_resource
                        .borrow_mut()
                        .set_app_resource(app::FileResource::UserManifest(Ptr::downgrade(
                            self.app.app_state().user_manifest(),
                        )));

                    return Ok(());
                } else if path == project_manifest.path() {
                    project_manifest.set_fs_resource(&fs_resource, app::DataResourceState::Valid);
                    fs_resource
                        .borrow_mut()
                        .set_app_resource(app::FileResource::ProjectManifest(Ptr::downgrade(
                            self.app.app_state().project_manifest(),
                        )));

                    return Ok(());
                }

                todo!();
                Ok(())
            }

            Action::Remove(resource) => {
                match resource {
                    action::FsResource::File(file) => {
                        if let Some(app_resource) = file.borrow().app_resource() {
                            match app_resource {
                                app::FileResource::UserManifest(manifest) => {
                                    let manifest = manifest.upgrade().unwrap();
                                    let mut manifest = manifest.borrow_mut();
                                    manifest.remove_fs_resource();
                                }
                                app::FileResource::ProjectManifest(manifest) => {
                                    let manifest = manifest.upgrade().unwrap();
                                    let mut manifest = manifest.borrow_mut();
                                    manifest.remove_fs_resource();
                                }
                                app::FileResource::ProjectProperties(properties) => {
                                    let properties = properties.upgrade().unwrap();
                                    let mut properties = properties.borrow_mut();
                                    properties.remove_fs_resource();
                                }
                                app::FileResource::ProjectSettings(settings) => {
                                    let settings = settings.upgrade().unwrap();
                                    let mut settings = settings.borrow_mut();
                                    settings.remove_fs_resource();
                                }
                                app::FileResource::AnalysisManifest(manifest) => {
                                    let manifest = manifest.upgrade().unwrap();
                                    let mut manifest = manifest.borrow_mut();
                                    manifest.remove_fs_resource();
                                }
                                app::FileResource::ContainerProperties(properties) => {
                                    let properties = properties.upgrade().unwrap();
                                    let mut properties = properties.borrow_mut();
                                    properties.remove_fs_resource();
                                }
                                app::FileResource::ContainerSettings(settings) => {
                                    let settings = settings.upgrade().unwrap();
                                    let mut settings = settings.borrow_mut();
                                    settings.remove_fs_resource();
                                }
                                app::FileResource::AssetManifest(manifest) => {
                                    let manifest = manifest.upgrade().unwrap();
                                    let mut manifest = manifest.borrow_mut();
                                    manifest.remove_fs_resource();
                                }
                                app::FileResource::Asset(asset) => {
                                    todo!();
                                }
                            }
                        }
                    }

                    action::FsResource::Folder(folder) => {
                        if let Some(app_resource) = folder.borrow().app_resource() {
                            match app_resource {
                                app::FolderResource::Project(project) => {
                                    let project = project.upgrade().unwrap();
                                    self.app
                                        .projects_mut()
                                        .retain(|prj| !Ptr::ptr_eq(&project, prj));
                                }

                                app::FolderResource::ProjectConfig(config) => {
                                    let project = self
                                        .app
                                        .find_resource_project(
                                            app::FolderResource::ProjectConfig(config).into(),
                                        )
                                        .unwrap();

                                    project.borrow_mut().remove_config();
                                }

                                app::FolderResource::Analyses(analyses) => {
                                    let project = self
                                        .app
                                        .find_resource_project(
                                            app::FolderResource::Analyses(analyses).into(),
                                        )
                                        .unwrap();

                                    project.borrow_mut().remove_analyses_folder_reference();
                                }

                                app::FolderResource::Data(data) => {
                                    let project = self
                                        .app
                                        .find_resource_project(
                                            app::FolderResource::Data(data).into(),
                                        )
                                        .unwrap();

                                    project.borrow_mut().remove_data_root();
                                }

                                app::FolderResource::Container(_) => todo!(),
                                app::FolderResource::ContainerConfig(_) => todo!(),
                            }
                        }
                    }
                }

                Ok(())
            }

            Action::Rename { resource, to } => match resource {
                action::FsResource::File(file) => {
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

                    Ok(())
                }

                action::FsResource::Folder(folder) => {
                    if let Some(app_resource) = folder.borrow().app_resource() {
                        match app_resource {
                            app::FolderResource::Project(_) => todo!(),
                            app::FolderResource::ProjectConfig(_) => todo!(),
                            app::FolderResource::Analyses(_) => todo!(),
                            app::FolderResource::Data(_) => todo!(),
                            app::FolderResource::Container(_) => todo!(),
                            app::FolderResource::ContainerConfig(_) => todo!(),
                        }
                    }

                    Ok(())
                }
            },

            Action::Move { resource, parent } => todo!(),
            Action::Copy { resource, parent } => todo!(),
            Action::Modify { file, kind } => match kind {
                action::ModifyKind::ManifestAdd(item) => todo!(),
                action::ModifyKind::ManifestRemove(index) => todo!(),
                action::ModifyKind::Corrupt => {
                    match file.borrow().app_resource().unwrap() {
                        app::FileResource::UserManifest(manifest) => {
                            let manifest = manifest.upgrade().unwrap();
                            let mut manifest = manifest.borrow_mut();
                            manifest.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::ProjectManifest(manifest) => {
                            let manifest = manifest.upgrade().unwrap();
                            let mut manifest = manifest.borrow_mut();
                            manifest.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::ProjectProperties(properties) => {
                            let properties = properties.upgrade().unwrap();
                            let mut properties = properties.borrow_mut();
                            properties.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::ProjectSettings(settings) => {
                            let settings = settings.upgrade().unwrap();
                            let mut settings = settings.borrow_mut();
                            settings.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::AnalysisManifest(manifest) => {
                            let manifest = manifest.upgrade().unwrap();
                            let mut manifest = manifest.borrow_mut();
                            manifest.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::ContainerProperties(properties) => {
                            let properties = properties.upgrade().unwrap();
                            let mut properties = properties.borrow_mut();
                            properties.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::ContainerSettings(settings) => {
                            let settings = settings.upgrade().unwrap();
                            let mut settings = settings.borrow_mut();
                            settings.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::AssetManifest(manifest) => {
                            let manifest = manifest.upgrade().unwrap();
                            let mut manifest = manifest.borrow_mut();
                            manifest.set_fs_resource(file, app::DataResourceState::Invalid)
                        }
                        app::FileResource::Asset(asset) => unreachable!(),
                    }
                    Ok(())
                }
                action::ModifyKind::Repair => todo!(),
                action::ModifyKind::Other => Ok(()),
            },
        }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        let (fs_state, fs_node_map) = self.fs.duplicate_with_app_references_and_map();
        let (mut app_state, _app_resource_map) = self.app.duplicate_with_fs_references_and_map();

        if let app::FsDataResource::Present { resource, state } = app_state
            .app_state()
            .user_manifest()
            .borrow()
            .fs_resource()
            .clone()
        {
            let resource_ptr = resource.upgrade().unwrap();
            let from_folder = self.fs.find_file_folder_by_ptr(&resource_ptr).unwrap();
            let to_folder = find_mapped_to(&from_folder, &fs_node_map).unwrap();

            let to_resource = to_folder
                .borrow()
                .file(resource_ptr.borrow().name())
                .unwrap();

            app_state
                .app_state()
                .user_manifest()
                .borrow_mut()
                .set_fs_resource(&to_resource, state);

            to_resource
                .borrow_mut()
                .set_app_resource(app::FileResource::UserManifest(Ptr::downgrade(
                    app_state.app_state().user_manifest(),
                )));
        }

        if let app::FsDataResource::Present { resource, state } = app_state
            .app_state()
            .project_manifest()
            .borrow()
            .fs_resource()
            .clone()
        {
            let resource_ptr = resource.upgrade().unwrap();
            let from_folder = self.fs.find_file_folder_by_ptr(&resource_ptr).unwrap();
            let to_folder = find_mapped_to(from_folder, &fs_node_map).unwrap();

            let to_resource = to_folder
                .borrow()
                .file(resource_ptr.borrow().name())
                .unwrap();

            app_state
                .app_state()
                .user_manifest()
                .borrow_mut()
                .set_fs_resource(&to_resource, state);

            to_resource
                .borrow_mut()
                .set_app_resource(app::FileResource::ProjectManifest(Ptr::downgrade(
                    app_state.app_state().project_manifest(),
                )));
        }

        for project in app_state.projects_mut().iter_mut() {
            if let app::FsResource::Present(resource) = project.borrow().fs_resource().clone() {
                let resource_ptr = resource.upgrade().unwrap();
                let project_folder = find_mapped_to(&resource_ptr, &fs_node_map).unwrap();

                project.borrow_mut().set_fs_resource(project_folder);

                project_folder
                    .borrow_mut()
                    .set_app_resource(app::FolderResource::Project(Ptr::downgrade(project)));
            }

            if let app::Resource::Present(config) = project.borrow().config() {
                let resource_ptr = config.borrow().fs_resource().upgrade().unwrap();
                let config_folder = find_mapped_to(&resource_ptr, &fs_node_map).unwrap();

                if let app::FsDataResource::Present { resource, state } =
                    config.borrow().properties().borrow().fs_resource().clone()
                {
                    let file_ptr = resource.upgrade().unwrap();
                    let file = config_folder
                        .borrow()
                        .file(file_ptr.borrow().name())
                        .unwrap();

                    assert_eq!(file.borrow().name(), constants::PROJECT_FILE);
                    *config.borrow().properties().borrow_mut() = match state {
                        app::DataResourceState::Valid => app::ProjectProperties::valid(&file),
                        app::DataResourceState::Invalid => app::ProjectProperties::invalid(&file),
                    };

                    file.borrow_mut()
                        .set_app_resource(app::FileResource::ProjectProperties(Ptr::downgrade(
                            config.borrow().properties(),
                        )));
                }

                if let app::FsDataResource::Present { resource, state } =
                    config.borrow().settings().borrow().fs_resource().clone()
                {
                    let file_ptr = resource.upgrade().unwrap();
                    let file = config_folder
                        .borrow()
                        .file(file_ptr.borrow().name())
                        .unwrap();

                    assert_eq!(file.borrow().name(), constants::PROJECT_SETTINGS_FILE);
                    *config.borrow().settings().borrow_mut() = match state {
                        app::DataResourceState::Valid => app::ProjectSettings::valid(&file),
                        app::DataResourceState::Invalid => app::ProjectSettings::invalid(&file),
                    };

                    file.borrow_mut()
                        .set_app_resource(app::FileResource::ProjectSettings(Ptr::downgrade(
                            config.borrow().settings(),
                        )));
                }

                if let app::FsDataResource::Present { resource, state } =
                    config.borrow().analyses().borrow().fs_resource().clone()
                {
                    let file_ptr = resource.upgrade().unwrap();
                    let file = config_folder
                        .borrow()
                        .file(file_ptr.borrow().name())
                        .unwrap();

                    assert_eq!(file.borrow().name(), constants::ANALYSES_FILE);
                    config
                        .borrow()
                        .analyses()
                        .borrow_mut()
                        .set_fs_resource(&file, state);

                    file.borrow_mut()
                        .set_app_resource(app::FileResource::AnalysisManifest(Ptr::downgrade(
                            config.borrow().analyses(),
                        )));
                }

                config.borrow_mut().set_fs_resource(config_folder);
                config_folder
                    .borrow_mut()
                    .set_app_resource(app::FolderResource::ProjectConfig(Ptr::downgrade(config)));
            }

            if let Some(analyses) = project.borrow_mut().analyses() {
                if let app::FsResource::Present(resource) = analyses.borrow().fs_resource() {
                    let folder_ptr = resource.upgrade().unwrap();
                    let folder = find_mapped_to(&folder_ptr, &fs_node_map).unwrap();
                    analyses.borrow_mut().set_fs_resource(folder)
                }
            }

            if let Some(graph) = project.borrow().data().borrow().graph() {
                for container in graph.nodes() {
                    let folder_ptr = container.borrow().fs_resource().upgrade().unwrap();
                    let container_folder = find_mapped_to(&folder_ptr, &fs_node_map).unwrap();

                    if let app::Resource::Present(resource) = container.borrow().data() {
                        let data = resource.borrow();
                        let config = data.config();
                        let folder_ptr = config.borrow().fs_resource().upgrade().unwrap();
                        let config_folder = find_mapped_to(&folder_ptr, &fs_node_map).unwrap();
                        assert_eq!(config_folder.borrow().name(), constants::APP_DIR);

                        if let app::FsDataResource::Present { resource, state } =
                            config.borrow().properties().borrow().fs_resource().clone()
                        {
                            let file_ptr = resource.upgrade().clone().unwrap();
                            let file = folder_ptr.borrow().file(file_ptr.borrow().name()).unwrap();
                            assert_eq!(file.borrow().name(), constants::CONTAINER_FILE);

                            config
                                .borrow()
                                .properties()
                                .borrow_mut()
                                .set_fs_resource(&file, state);

                            file.borrow_mut().set_app_resource(
                                app::FileResource::ContainerProperties(Ptr::downgrade(
                                    config.borrow().properties(),
                                )),
                            );
                        }

                        if let app::FsDataResource::Present { resource, state } =
                            config.borrow().settings().borrow().fs_resource().clone()
                        {
                            let file_ptr = resource.upgrade().clone().unwrap();
                            let file = folder_ptr.borrow().file(file_ptr.borrow().name()).unwrap();
                            assert_eq!(file.borrow().name(), constants::CONTAINER_SETTINGS_FILE);

                            config
                                .borrow()
                                .settings()
                                .borrow_mut()
                                .set_fs_resource(&file, state);

                            file.borrow_mut().set_app_resource(
                                app::FileResource::ContainerSettings(Ptr::downgrade(
                                    config.borrow().settings(),
                                )),
                            );
                        }

                        if let app::FsDataResource::Present { resource, state } =
                            config.borrow().assets().borrow().fs_resource().clone()
                        {
                            let file_ptr = resource.upgrade().clone().unwrap();
                            let file = folder_ptr.borrow().file(file_ptr.borrow().name()).unwrap();
                            assert_eq!(file.borrow().name(), constants::ASSETS_FILE);

                            config
                                .borrow()
                                .assets()
                                .borrow_mut()
                                .set_fs_resource(&file, state);

                            file.borrow_mut()
                                .set_app_resource(app::FileResource::AssetManifest(
                                    Ptr::downgrade(config.borrow().assets()),
                                ));
                        }

                        config.borrow_mut().set_fs_resource(config_folder);
                        config_folder.borrow_mut().set_app_resource(
                            app::FolderResource::ContainerConfig(Ptr::downgrade(config)),
                        );
                    }

                    container.borrow_mut().set_fs_resource(container_folder);
                    container_folder
                        .borrow_mut()
                        .set_app_resource(app::FolderResource::Container(Ptr::downgrade(
                            container,
                        )));
                }
            }
        }

        Self {
            fs: fs_state,
            app: app_state,
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
