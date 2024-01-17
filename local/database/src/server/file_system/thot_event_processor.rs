//! Processes [`FileSystemEvent`]s into [`ThotEvent`]s.
use super::event::{file_system, thot};
use crate::error::{Error, Result};
use crate::server::store::ContainerTree;
use crate::server::Database;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thot_core::error::{Error as CoreError, Project as CoreProjectError, ResourceError};
use thot_core::project::ScriptLang;
use thot_core::types::{ResourceId, ResourcePath};
use thot_local::error::{Error as LocalError, ProjectError};
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
    pub fn process_file_system_events_to_thot_events(
        &mut self,
        events: &Vec<file_system::Event>,
    ) -> Vec<thot::Event> {
        let mut thot_events = Vec::with_capacity(events.len());
        for event in events {
            let processed_events = match event {
                file_system::Event::File(file_system::File::Created(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_file_created(&path)
                        .unwrap()
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::File(file_system::File::Removed(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_file_removed(&path)
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::File(file_system::File::Moved { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_file_moved(&from, to)
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::File(file_system::File::Renamed { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_file_renamed(&from, to)
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::File(file_system::File::Modified(path)) => {
                    vec![]
                }

                file_system::Event::Folder(file_system::Folder::Created(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_folder_created(&path)
                        .unwrap()
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::Folder(file_system::Folder::Removed(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_folder_removed(&path)
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::Folder(file_system::Folder::Moved { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_folder_moved(&from, to)
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::Folder(file_system::Folder::Renamed { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_folder_renamed(&from, to)
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::Folder(file_system::Folder::Modified(path)) => {
                    vec![]
                }

                file_system::Event::Any(file_system::Any::Created(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_any_created(&path)
                        .unwrap()
                        .into_iter()
                        .map(|event| event.into())
                        .collect()
                }

                file_system::Event::Any(file_system::Any::Removed(path)) => {
                    let path = normalize_path_root(path);
                    match self.ensure_project_resources_loaded(&path) {
                        Ok(_) => self.handle_any_removed(&path),
                        Err(Error::Local(LocalError::ProjectError(
                            ProjectError::PathNotInProject(_),
                        ))) => self.handle_any_removed_path_not_in_project(path),
                        Err(_) => todo!(),
                    }
                }
            };

            thot_events.extend(processed_events);
        }

        thot_events
    }

    fn handle_file_created(&self, path: &PathBuf) -> Result<Vec<thot::Event>> {
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
                    return Ok(vec![thot::Script::Created(path.clone()).into()]);
                }

                return Ok(vec![]);
            }
        }

        // ignore if asset
        if let Some(asset) = self.store.get_path_asset_id_canonical(&path).unwrap() {
            return Ok(vec![thot::Asset::FileCreated(asset.clone()).into()]);
        }

        // handle new
        return Ok(vec![thot::File::Created(path.into()).into()]);
    }

    fn handle_file_removed(&self, path: &PathBuf) -> Vec<thot::Event> {
        let project = self.project_by_resource_path(path).unwrap();

        // script
        let scripts = self.store.get_project_scripts(&project.rid).unwrap();
        let script_path = path
            .strip_prefix(project.analysis_root_path().unwrap())
            .unwrap();

        let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
        if let Some(script) = scripts.by_path(&script_path) {
            return vec![thot::Script::Removed(script.rid.clone()).into()];
        }

        // analysis
        if let Some(asset) = self.store.get_path_asset_id(path).cloned() {
            return vec![thot::Asset::Removed(asset).into()];
        };

        vec![]
    }

    /// Handles a moved file
    /// A moved file has the same file name, but its base directory has changed.
    fn handle_file_moved(&self, from: &PathBuf, to: &PathBuf) -> Vec<thot::Event> {
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
                    return vec![thot::Asset::Moved {
                        asset,
                        path: to.clone(),
                    }
                    .into()];
                }

                return vec![thot::File::Created(to.clone()).into()];
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
                    return vec![thot::Script::Moved {
                        script: script.rid.clone(),
                        path: to.clone(),
                    }
                    .into()];
                }

                vec![thot::Script::Created(to.clone()).into()]
            }

            (Location::None, Location::Data) => {
                vec![thot::File::Created(to.clone()).into()]
            }

            (Location::Data, Location::None) => {
                let asset = self
                    .store
                    .get_path_asset_id_canonical(from)
                    .unwrap_or_else(|_| self.store.get_path_asset_id(from));

                if let Some(asset) = asset {
                    return vec![thot::Asset::Removed(asset.clone()).into()];
                }

                vec![]
            }

            (Location::None, Location::Analysis) => {
                vec![thot::Script::Created(to.clone()).into()]
            }

            (Location::Analysis, Location::None) => {
                let from_script_path = from
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let from_script_path = ResourcePath::new(from_script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if let Some(script) = scripts.by_path(&from_script_path) {
                    return vec![thot::Script::Removed(script.rid.clone()).into()];
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
                    events.push(thot::Asset::Removed(asset.clone()).into());
                }

                let Some(ext) = to.extension() else {
                    return events;
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if ScriptLang::supported_extensions().contains(&ext) {
                    events.push(thot::Script::Created(to.clone()).into());
                }

                events
            }

            (Location::Analysis, Location::Data) => {
                let mut events = vec![thot::File::Created(to.clone()).into()];
                let from_script_path = from
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let from_script_path = ResourcePath::new(from_script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if let Some(script) = scripts.by_path(&from_script_path) {
                    events.push(thot::Script::Removed(script.rid.clone()).into());
                }

                events
            }

            (Location::None, Location::None) => vec![],
        }
    }

    fn handle_file_renamed(&self, from: &PathBuf, to: &PathBuf) -> Vec<thot::Event> {
        if let Some(asset) = self.store.get_path_asset_id(&from).cloned() {
            return vec![thot::Asset::Moved {
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
                    return vec![thot::Script::Moved {
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
                    return vec![thot::Script::Created(to.clone()).into()];
                }

                return vec![];
            }
        }

        vec![]
    }

    fn handle_folder_created(&self, path: &PathBuf) -> Result<Vec<thot::Event>> {
        // ignore thot folder
        if path
            .components()
            .any(|seg| seg.as_os_str() == thot_local::common::thot_dir().as_os_str())
        {
            return Ok(vec![]);
        }

        // ignore graph root
        let project = self.project_by_resource_path(path)?;
        if project.data_root.is_some() {
            let path = fs::canonicalize(&path).unwrap();
            if path == project.data_root_path().unwrap() {
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
                    return Ok(vec![thot::Graph::Inserted(graph).into()]);
                };

                if loaded_container.base_path().exists() {
                    Ok(vec![thot::Graph::Copied(graph).into()])
                } else {
                    Ok(vec![thot::Graph::Moved {
                        root: graph.root().clone(),
                        path: path.clone(),
                    }
                    .into()])
                }
            }

            Err(PartialLoad { errors, graph }) => match errors.get(path) {
                Some(ContainerTreeLoaderError::Dir(err)) if err == &io::ErrorKind::NotFound => {
                    return Ok(vec![thot::Folder::Created(path.clone()).into()]);
                }

                Some(ContainerTreeLoaderError::Load(ContainerLoaderError::NotResource)) => {
                    return Ok(vec![thot::Folder::Created(path.clone()).into()]);
                }

                _ => {
                    let graph = graph.map(|graph| ContainerTreeTransformer::local_to_core(&graph));
                    return Err(Error::LoadPartial { errors, graph });
                }
            },
        }
    }

    fn handle_folder_removed(&self, path: &PathBuf) -> Vec<thot::Event> {
        let Some(container) = self.store.get_path_container(path).cloned() else {
            return vec![];
        };

        vec![thot::Graph::Removed(container).into()]
    }

    fn handle_folder_moved(&self, from: &PathBuf, to: &PathBuf) -> Vec<thot::Graph> {
        let Some(root) = self.store.get_path_container(from).cloned() else {
            return vec![];
        };

        vec![thot::Graph::Moved {
            root,
            path: to.clone(),
        }]
    }

    fn handle_folder_renamed(&self, from: &PathBuf, to: &PathBuf) -> Vec<thot::Graph> {
        let Some(container) = self.store.get_path_container(from).cloned() else {
            return vec![];
        };

        vec![thot::Graph::Moved {
            root: container,
            path: to.clone(),
        }]
    }

    fn handle_any_created(&self, path: &PathBuf) -> Result<Vec<thot::Event>> {
        if path.is_file() {
            self.handle_file_created(path)
        } else if path.is_dir() {
            self.handle_folder_created(path)
        } else {
            Ok(vec![])
        }
    }

    fn handle_any_removed(&self, path: &PathBuf) -> Vec<thot::Event> {
        if let Some(container) = self.store.get_path_container(&path).cloned() {
            return vec![thot::Graph::Removed(container).into()];
        }

        if let Some(asset) = self.store.get_path_asset_id(&path).cloned() {
            return vec![thot::Asset::Removed(asset).into()];
        }

        let project = self.project_by_resource_path(&path).unwrap();
        let scripts = self.store.get_project_scripts(&project.rid).unwrap();
        if let Ok(script_path) = path.strip_prefix(project.analysis_root_path().unwrap()) {
            let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
            if let Some(script) = scripts.by_path(&script_path) {
                return vec![thot::Script::Removed(script.rid.clone()).into()];
            }
        }

        vec![]
    }

    fn handle_any_removed_path_not_in_project(&self, path: impl AsRef<Path>) -> Vec<thot::Event> {
        let path = path.as_ref();
        if let Some(project) = self.store.get_path_project(path) {
            return vec![thot::Project::Removed(project.clone()).into()];
        }

        if let Some(file_name) = path.file_name() {
            if file_name == thot_local::common::thot_dir() {
                if let Some(project_path) = path.parent() {
                    if let Some(project) = self.store.get_path_project(project_path) {
                        return vec![thot::Project::Removed(project.clone()).into()];
                    }
                }
            }
        }

        vec![]
    }

    /// Get a `Project` by a path within it.
    fn project_by_resource_path(&self, path: impl AsRef<Path>) -> Result<&LocalProject> {
        let path = path.as_ref();
        let project_path = project::project_root_path(path)?;
        let Some(project) = self
            .store
            .get_path_project_canonical(&project_path)
            .unwrap()
        else {
            return Err(LocalError::ProjectError(ProjectError::PathNotInProject(
                path.to_path_buf(),
            ))
            .into());
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
        let project = project_root_path(path.as_ref())?;
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
