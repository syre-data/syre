//! Processes file system events.
use super::event::{app, file_system};
use crate::error::{Error, Result};
use crate::server::store::ContainerTree;
use crate::server::Database;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thot_core::error::{Error as CoreError, Project as CoreProjectError, ResourceError};
use thot_core::project::ScriptLang;
use thot_core::types::{ResourceId, ResourcePath};
use thot_local::error::{Error as LocalError, Project as ProjectError};
use thot_local::graph::ContainerTreeTransformer;
use thot_local::loader::error::container::Error as ContainerLoaderError;
use thot_local::loader::error::tree::Error as ContainerTreeLoaderError;
use thot_local::loader::tree::incremental::{
    Loader as ContainerTreeIncrementalLoader, PartialLoad,
};
use thot_local::loader::tree::Loader as ContainerTreeLoader;
use thot_local::project::project;
use thot_local::project::project::project_root_path;
use thot_local::project::resources::{Project as LocalProject, Scripts as ProjectScripts};

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn process_file_system_event(&mut self, event: &file_system::Event) -> Result {
        let app_events = self.process_file_system_event_to_app_events(event);
        for app_event in app_events {
            if let Err(err) = self.handle_app_event(&app_event) {
                tracing::debug!(?app_event, ?err);
                return Err(err);
            }
        }

        Ok(())
    }

    fn handle_app_event(&mut self, event: &app::Event) -> Result {
        tracing::debug!(?event);
        match event {
            app::Event::Project(event) => self.handle_thot_event_project(event)?,
            app::Event::Graph(event) => self.handle_thot_event_graph(event)?,
            app::Event::Container(event) => self.handle_thot_event_container(event)?,
            app::Event::Asset(event) => self.handle_thot_event_asset(event)?,
            app::Event::Script(event) => self.handle_thot_event_script(event)?,
            app::Event::Folder(event) => self.handle_thot_event_folder(event)?,
            app::Event::File(event) => self.handle_thot_event_file(event)?,
        }

        Ok(())
    }

    fn process_file_system_event_to_app_events(
        &mut self,
        event: &file_system::Event,
    ) -> Vec<app::Event> {
        tracing::debug!(?event);
        match event.kind() {
            file_system::EventKind::File(file_system::File::Created(path)) => {
                let path = normalize_path_root(path);
                self.ensure_project_resources_loaded(&path).unwrap();
                self.handle_file_created(&path)
                    .unwrap()
                    .into_iter()
                    .map(|event| event.into())
                    .collect()
            }

            file_system::EventKind::File(file_system::File::Removed(path)) => {
                let path = normalize_path_root(path);
                self.ensure_project_resources_loaded(&path).unwrap();
                self.handle_file_removed(&path)
                    .into_iter()
                    .map(|event| event.into())
                    .collect()
            }

            file_system::EventKind::File(file_system::File::Moved { from, to }) => {
                let from = normalize_path_root(from);
                self.ensure_project_resources_loaded(&from).unwrap();
                self.ensure_project_resources_loaded(&to).unwrap();
                self.handle_file_moved(&from, to)
                    .into_iter()
                    .map(|event| event.into())
                    .collect()
            }

            file_system::EventKind::File(file_system::File::Renamed { from, to }) => {
                let from = normalize_path_root(from);
                self.ensure_project_resources_loaded(&from).unwrap();
                self.ensure_project_resources_loaded(&to).unwrap();
                self.handle_file_renamed(&from, to)
                    .into_iter()
                    .map(|event| event.into())
                    .collect()
            }

            file_system::EventKind::File(file_system::File::Modified(path)) => {
                vec![]
            }

            file_system::EventKind::Folder(file_system::Folder::Created(path)) => {
                let path = normalize_path_root(path);
                self.ensure_project_resources_loaded(&path).unwrap();
                self.handle_folder_created(&path)
                    .unwrap()
                    .into_iter()
                    .map(|event| event.into())
                    .collect()
            }

            file_system::EventKind::Folder(file_system::Folder::Removed(path)) => {
                let path = normalize_path_root(path);
                self.ensure_project_resources_loaded(&path).unwrap();
                self.handle_folder_removed(&path)
                    .into_iter()
                    .map(|event| event.into())
                    .collect()
            }

            file_system::EventKind::Folder(file_system::Folder::Moved { from, to }) => {
                let from = normalize_path_root(from);
                self.ensure_project_resources_loaded(&from).unwrap();
                self.ensure_project_resources_loaded(&to).unwrap();
                self.handle_folder_moved(&from, to)
                    .into_iter()
                    .map(|event| event.into())
                    .collect()
            }

            file_system::EventKind::Folder(file_system::Folder::Renamed { from, to }) => {
                let from = normalize_path_root(from);
                let from_loaded = self.ensure_project_resources_loaded(&from);
                let to_loaded = self.ensure_project_resources_loaded(&to);
                match (from_loaded, to_loaded) {
                    (Ok(_), Ok(_)) => self
                        .handle_folder_renamed(&from, to)
                        .into_iter()
                        .map(|event| event.into())
                        .collect(),

                    (
                        Err(Error::Local(LocalError::Project(ProjectError::PathNotInProject(_)))),
                        Err(Error::Core(CoreError::ResourceError(ResourceError::DoesNotExist(_)))),
                    ) => self
                        .handle_project_folder_renamed(&from, to)
                        .into_iter()
                        .map(|event| event.into())
                        .collect(),

                    (from_err, to_err) => {
                        tracing::debug!(?from_err, ?to_err);
                        vec![]
                    }
                }
            }

            file_system::EventKind::Folder(file_system::Folder::Modified(_path)) => {
                vec![]
            }

            file_system::EventKind::Any(file_system::Any::Removed(path)) => {
                let path = normalize_path_root(path);
                match self.ensure_project_resources_loaded(&path) {
                    Ok(_) => self.handle_any_removed(&path),
                    Err(Error::Local(LocalError::Project(ProjectError::PathNotInProject(_)))) => {
                        self.handle_any_removed_path_not_in_project(path)
                    }
                    Err(_) => todo!(),
                }
            }
        }
    }

    fn handle_file_created(&self, path: &PathBuf) -> Result<Vec<app::Event>> {
        // ignore thot folder
        if path
            .components()
            .any(|seg| seg.as_os_str() == thot_local::common::thot_dir().as_os_str())
        {
            return Ok(vec![]);
        }

        // analysis root
        let project = self.project_by_resource_path(&path)?;
        if let Some(analysis_root) = project.analysis_root_path().as_ref() {
            if let Ok(script_path) = path.strip_prefix(analysis_root) {
                let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if scripts.contains_path(&script_path) {
                    return Ok(vec![]);
                }

                let Some(ext) = path.extension() else {
                    return Ok(vec![]);
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if ScriptLang::supported_extensions().contains(&ext) {
                    return Ok(vec![app::Script::Created(path.clone()).into()]);
                }

                return Ok(vec![]);
            }
        }

        // ignore if asset
        if let Some(asset) = self.store.get_path_asset_id_canonical(&path).unwrap() {
            return Ok(vec![app::Asset::FileCreated(asset.clone()).into()]);
        }

        // handle new
        return Ok(vec![app::File::Created(path.into()).into()]);
    }

    fn handle_file_removed(&self, path: &PathBuf) -> Vec<app::Event> {
        let project = self.project_by_resource_path(path).unwrap();

        // script
        let scripts = self.store.get_project_scripts(&project.rid).unwrap();
        let script_path = path
            .strip_prefix(project.analysis_root_path().unwrap())
            .unwrap();

        let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
        if let Some(script) = scripts.by_path(&script_path) {
            return vec![app::Script::Removed(script.rid.clone()).into()];
        }

        // analysis
        if let Some(asset) = self.store.get_path_asset_id(path).cloned() {
            return vec![app::Asset::Removed(asset).into()];
        };

        vec![]
    }

    /// Handles a moved file
    /// A moved file has the same file name, but its base directory has changed.
    fn handle_file_moved(&self, from: &PathBuf, to: &PathBuf) -> Vec<app::Event> {
        #[derive(Debug)]
        enum Location {
            Data,
            Analysis,
            None,
        }

        fn get_path_resource_type(project: &LocalProject, path: &PathBuf) -> Location {
            if path.starts_with(project.data_root_path().unwrap()) {
                return Location::Data;
            } else if path.starts_with(project.analysis_root_path().unwrap()) {
                return Location::Analysis;
            }

            Location::None
        }

        let project = self.project_by_resource_path(&from).unwrap();
        let from_type = get_path_resource_type(project, from);
        let to_type = get_path_resource_type(project, to);
        tracing::debug!(?from_type, ?to_type);

        match (from_type, to_type) {
            (Location::Data, Location::Data) => {
                if let Some(asset) = self.store.get_path_asset_id(&from).cloned() {
                    return vec![app::Asset::Moved {
                        asset,
                        path: to.clone(),
                    }
                    .into()];
                }

                return vec![app::File::Created(to.clone()).into()];
            }

            (Location::Analysis, Location::Analysis) => {
                let Some(ext) = to.extension() else {
                    return vec![];
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if !ScriptLang::supported_extensions().contains(&ext) {
                    return vec![];
                }

                let from_script_path = from
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let from_script_path = ResourcePath::new(from_script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if let Some(script) = scripts.by_path(&from_script_path) {
                    return vec![app::Script::Moved {
                        script: script.rid.clone(),
                        path: to.clone(),
                    }
                    .into()];
                }

                vec![app::Script::Created(to.clone()).into()]
            }

            (Location::None, Location::Data) => {
                vec![app::File::Created(to.clone()).into()]
            }

            (Location::Data, Location::None) => {
                let asset = self
                    .store
                    .get_path_asset_id_canonical(from)
                    .unwrap_or_else(|_| self.store.get_path_asset_id(from));

                if let Some(asset) = asset {
                    return vec![app::Asset::Removed(asset.clone()).into()];
                }

                vec![]
            }

            (Location::None, Location::Analysis) => {
                vec![app::Script::Created(to.clone()).into()]
            }

            (Location::Analysis, Location::None) => {
                let from_script_path = from
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let from_script_path = ResourcePath::new(from_script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if let Some(script) = scripts.by_path(&from_script_path) {
                    return vec![app::Script::Removed(script.rid.clone()).into()];
                }

                vec![]
            }

            (Location::Data, Location::Analysis) => {
                let mut events = vec![];
                let asset = self
                    .store
                    .get_path_asset_id_canonical(from)
                    .unwrap_or_else(|_| self.store.get_path_asset_id(from));

                if let Some(asset) = asset {
                    events.push(app::Asset::Removed(asset.clone()).into());
                }

                let Some(ext) = to.extension() else {
                    return events;
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if ScriptLang::supported_extensions().contains(&ext) {
                    events.push(app::Script::Created(to.clone()).into());
                }

                events
            }

            (Location::Analysis, Location::Data) => {
                let mut events = vec![app::File::Created(to.clone()).into()];
                let from_script_path = from
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let from_script_path = ResourcePath::new(from_script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if let Some(script) = scripts.by_path(&from_script_path) {
                    events.push(app::Script::Removed(script.rid.clone()).into());
                }

                events
            }

            (Location::None, Location::None) => vec![],
        }
    }

    fn handle_file_renamed(&self, from: &PathBuf, to: &PathBuf) -> Vec<app::Event> {
        if let Some(asset) = self.store.get_path_asset_id(&from).cloned() {
            return vec![app::Asset::Moved {
                asset,
                path: to.clone(),
            }
            .into()];
        };

        let project = self.project_by_resource_path(&from).unwrap();
        if let Some(analysis_root) = project.analysis_root_path().as_ref() {
            let script_path = from.strip_prefix(analysis_root).unwrap();
            let scripts = self.store.get_project_scripts(&project.rid).unwrap();
            for script in scripts.values() {
                if script.path.as_path() == script_path {
                    return vec![app::Script::Moved {
                        script: script.rid.clone(),
                        path: to.clone(),
                    }
                    .into()];
                }
            }

            if let Ok(script_path) = to.strip_prefix(analysis_root) {
                let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if scripts.contains_path(&script_path) {
                    return vec![];
                }

                let Some(ext) = to.extension() else {
                    return vec![];
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if ScriptLang::supported_extensions().contains(&ext) {
                    return vec![app::Script::Created(to.clone()).into()];
                }

                return vec![];
            }
        }

        vec![]
    }

    fn handle_folder_created(&self, path: &PathBuf) -> Result<Vec<app::Event>> {
        // ignore app folder
        if path
            .components()
            .any(|seg| seg.as_os_str() == thot_local::common::thot_dir().as_os_str())
        {
            return Ok(vec![]);
        }

        // ignore graph root and above
        let project = self.project_by_resource_path(path)?;
        if let Some(data_root) = project.data_root_path() {
            let path = fs::canonicalize(&path).unwrap();
            if !path.parent().unwrap().starts_with(data_root) {
                return Ok(vec![]);
            }
        };

        // ignore if registered container
        if self
            .store
            .get_path_container_canonical(&path)
            .unwrap()
            .is_some()
        {
            return Ok(vec![]);
        }

        // handle if unregistered container
        match ContainerTreeIncrementalLoader::load(path) {
            Ok(graph) => {
                let Some(loaded_container) = self.store.get_container(graph.root()) else {
                    return Ok(vec![app::Graph::Inserted(graph).into()]);
                };

                if loaded_container.base_path().exists() {
                    Ok(vec![app::Graph::Copied(graph).into()])
                } else {
                    Ok(vec![app::Graph::Moved {
                        root: graph.root().clone(),
                        path: path.clone(),
                    }
                    .into()])
                }
            }

            Err(PartialLoad { errors, graph }) => match errors.get(path) {
                Some(ContainerTreeLoaderError::Dir(err)) if err == &io::ErrorKind::NotFound => {
                    return Ok(vec![app::Folder::Created(path.clone()).into()]);
                }

                Some(ContainerTreeLoaderError::Load(ContainerLoaderError::NotResource)) => {
                    return Ok(vec![app::Folder::Created(path.clone()).into()]);
                }

                _ => {
                    let graph = graph.map(|graph| ContainerTreeTransformer::local_to_core(&graph));
                    return Err(Error::LoadPartial { errors, graph });
                }
            },
        }
    }

    fn handle_folder_removed(&self, path: &PathBuf) -> Vec<app::Event> {
        let Some(container) = self.store.get_path_container(path).cloned() else {
            return vec![];
        };

        vec![app::Graph::Removed(container).into()]
    }

    fn handle_folder_moved(&self, from: &PathBuf, to: &PathBuf) -> Vec<app::Graph> {
        let Some(root) = self.store.get_path_container(from).cloned() else {
            return vec![];
        };

        vec![app::Graph::Moved {
            root,
            path: to.clone(),
        }]
    }

    fn handle_folder_renamed(&self, from: &PathBuf, to: &PathBuf) -> Vec<app::Graph> {
        let Some(container) = self.store.get_path_container(from).cloned() else {
            return vec![];
        };

        vec![app::Graph::Moved {
            root: container,
            path: to.clone(),
        }]
    }

    fn handle_project_folder_renamed(&self, from: &PathBuf, to: &PathBuf) -> Vec<app::Project> {
        let Some(project) = self.store.get_path_project(from).cloned() else {
            return vec![];
        };

        vec![app::Project::Moved {
            project,
            path: to.clone(),
        }]
    }

    fn handle_any_removed(&self, path: &PathBuf) -> Vec<app::Event> {
        if let Some(container) = self.store.get_path_container(&path).cloned() {
            return vec![app::Graph::Removed(container).into()];
        }

        if let Some(asset) = self.store.get_path_asset_id(&path).cloned() {
            return vec![app::Asset::Removed(asset).into()];
        }

        let project = self.project_by_resource_path(&path).unwrap();
        let scripts = self.store.get_project_scripts(&project.rid).unwrap();
        if let Ok(script_path) = path.strip_prefix(project.analysis_root_path().unwrap()) {
            let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
            if let Some(script) = scripts.by_path(&script_path) {
                return vec![app::Script::Removed(script.rid.clone()).into()];
            }
        }

        vec![]
    }

    fn handle_any_removed_path_not_in_project(&self, path: impl AsRef<Path>) -> Vec<app::Event> {
        let path = path.as_ref();
        if let Some(project) = self.store.get_path_project(path) {
            return vec![app::Project::Removed(project.clone()).into()];
        }

        if let Some(file_name) = path.file_name() {
            if file_name == thot_local::common::thot_dir() {
                if let Some(project_path) = path.parent() {
                    if let Some(project) = self.store.get_path_project(project_path) {
                        return vec![app::Project::Removed(project.clone()).into()];
                    }
                }
            }
        }

        vec![]
    }

    /// Get a `Project` by a path within it.
    fn project_by_resource_path(&self, path: impl AsRef<Path>) -> Result<&LocalProject> {
        let path = path.as_ref();
        let Some(project_path) = project::project_root_path(path) else {
            return Err(
                LocalError::Project(ProjectError::PathNotInProject(path.to_path_buf())).into(),
            );
        };

        let Some(project) = self
            .store
            .get_path_project_canonical(&project_path)
            .unwrap()
        else {
            return Err(
                LocalError::Project(ProjectError::PathNotInProject(path.to_path_buf())).into(),
            );
        };

        let Some(project) = self.store.get_project(project) else {
            return Err(Error::Database("Project not loaded".into()));
        };

        Ok(project)
    }

    /// Ensures all a `Project`'s resources are loaded.
    ///
    /// # Arguments
    /// 1. `path`: Path to a resource within the project.
    fn ensure_project_resources_loaded(&mut self, path: impl AsRef<Path>) -> Result {
        let path = path.as_ref();
        let Some(project) = project_root_path(path) else {
            return Err(
                LocalError::Project(ProjectError::PathNotInProject(path.to_path_buf())).into(),
            );
        };

        let Some(project) = self
            .store
            .get_path_project_canonical(project.as_ref())
            .unwrap()
            .cloned()
        else {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "`Project` not loaded",
            ))
            .into());
        };

        self.ensure_project_graph_loaded(&project)?;
        self.ensure_project_scripts_loaded(&project)?;
        Ok(())
    }

    /// Ensures a `Project` resource's graph is loaded.
    fn ensure_project_graph_loaded(&mut self, project: &ResourceId) -> Result {
        let project = self.store.get_project(project).unwrap();
        let Some(data_root) = project.data_root.as_ref() else {
            return Err(
                CoreError::Project(CoreProjectError::misconfigured("data root not set")).into(),
            );
        };

        if self.store.is_project_graph_loaded(&project.rid) {
            return Ok(());
        }

        let path = project.base_path().join(data_root);
        let graph: ContainerTree = ContainerTreeLoader::load(&path)?;
        self.store
            .insert_project_graph_canonical(project.rid.clone(), graph)?;

        Ok(())
    }

    /// Loads a `Project`'s `Scripts`.
    ///
    /// # Arguments
    /// 1. `Project`'s id.
    fn ensure_project_scripts_loaded(&mut self, project: &ResourceId) -> Result {
        if self.store.are_project_scripts_loaded(project) {
            return Ok(());
        }

        let project = self.store.get_project(project).unwrap();
        let scripts = ProjectScripts::load_from(project.base_path())?;
        self.store
            .insert_project_scripts(project.rid.clone(), scripts);

        Ok(())
    }
}

/// If on Windows, convert to UNC if needed.
/// Otherwise, returns the given path.
fn normalize_path_root(path: impl Into<PathBuf>) -> PathBuf {
    if cfg!(target_os = "windows") {
        thot_local::common::ensure_windows_unc(path)
    } else {
        path.into()
    }
}
