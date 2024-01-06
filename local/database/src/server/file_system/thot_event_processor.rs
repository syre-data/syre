//! Processes [`FileSystemEvent`]s into [`ThotEvent`]s.
use super::event::{file_system, thot};
use crate::error::{Error, Result};
use crate::server::store::ContainerTree;
use crate::server::Database;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thot_core::error::{Error as CoreError, ProjectError as CoreProjectError, ResourceError};
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
            let thot_event = match event {
                file_system::Event::File(file_system::File::Created(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_file_created(&path)
                        .unwrap()
                        .map(|event| event.into())
                }

                file_system::Event::File(file_system::File::Removed(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_file_removed(&path).map(|event| event.into())
                }

                file_system::Event::File(file_system::File::Moved { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_file_moved(&from, to).map(|event| event.into())
                }

                file_system::Event::File(file_system::File::Renamed { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_file_renamed(&from, to)
                        .map(|event| event.into())
                }

                file_system::Event::Folder(file_system::Folder::Created(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_folder_created(&path)
                        .unwrap()
                        .map(|event| event.into())
                }

                file_system::Event::Folder(file_system::Folder::Removed(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_folder_removed(&path).map(|event| event.into())
                }

                file_system::Event::Folder(file_system::Folder::Moved { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_folder_moved(&from, to)
                        .map(|event| event.into())
                }

                file_system::Event::Folder(file_system::Folder::Renamed { from, to }) => {
                    let from = normalize_path_root(from);
                    self.ensure_project_resources_loaded(&from).unwrap();
                    self.ensure_project_resources_loaded(&to).unwrap();
                    self.handle_folder_renamed(&from, to)
                        .map(|event| event.into())
                }

                file_system::Event::Any(file_system::Any::Created(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_any_created(&path)
                        .unwrap()
                        .map(|event| event.into())
                }

                file_system::Event::Any(file_system::Any::Removed(path)) => {
                    let path = normalize_path_root(path);
                    self.ensure_project_resources_loaded(&path).unwrap();
                    self.handle_any_removed(&path)
                }
            };

            if let Some(thot_event) = thot_event {
                thot_events.push(thot_event);
            }
        }

        thot_events
    }

    fn handle_file_created(&self, path: &PathBuf) -> Result<Option<thot::Event>> {
        // ignore thot folder
        if path
            .components()
            .any(|seg| seg.as_os_str() == thot_local::common::thot_dir().as_os_str())
        {
            return Ok(None);
        }

        // analysis root
        let project = self.project_by_resource_path(&path)?;
        if let Some(analysis_root) = project.analysis_root_path().as_ref() {
            if let Ok(script_path) = path.strip_prefix(analysis_root) {
                let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if scripts.contains_path(&script_path) {
                    return Ok(None);
                }

                let Some(ext) = path.extension() else {
                    return Ok(None);
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if ScriptLang::supported_extensions().contains(&ext) {
                    return Ok(Some(thot::Script::Created(path.clone()).into()));
                }

                return Ok(None);
            }
        }

        // ignore if asset
        if let Some(asset) = self.store.get_path_asset_id_canonical(&path).unwrap() {
            return Ok(Some(thot::Asset::FileCreated(asset.clone()).into()));
        }

        // handle new
        return Ok(Some(thot::File::Created(path.into()).into()));
    }

    fn handle_file_removed(&self, path: &PathBuf) -> Option<thot::Event> {
        let project = self.project_by_resource_path(path).unwrap();

        // script
        let scripts = self.store.get_project_scripts(&project.rid).unwrap();
        let script_path = path
            .strip_prefix(project.analysis_root_path().unwrap())
            .unwrap();

        let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
        if let Some(script) = scripts.by_path(&script_path) {
            return Some(thot::Script::Removed(script.rid.clone()).into());
        }

        // analysis
        if let Some(asset) = self.store.get_path_asset_id(path).cloned() {
            return Some(thot::Asset::Removed(asset).into());
        };

        None
    }

    /// Handles a moved file
    /// A moved file has the same file name, but its base directory has changed.
    fn handle_file_moved(&self, from: &PathBuf, to: &PathBuf) -> Option<thot::Event> {
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

        match (from_type, to_type) {
            (Location::Data, Location::Data) => {
                if let Some(asset) = self.store.get_path_asset_id(&from).cloned() {
                    return Some(
                        thot::Asset::Moved {
                            asset,
                            path: to.clone(),
                        }
                        .into(),
                    );
                }

                return Some(thot::File::Created(to.clone()).into());
            }

            (Location::Analysis, Location::Analysis) => {
                let Some(ext) = to.extension() else {
                    return None;
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if !ScriptLang::supported_extensions().contains(&ext) {
                    return None;
                }

                let from_script_path = from
                    .strip_prefix(project.analysis_root_path().unwrap())
                    .unwrap();

                let from_script_path = ResourcePath::new(from_script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if let Some(script) = scripts.by_path(&from_script_path) {
                    return Some(
                        thot::Script::Moved {
                            script: script.rid.clone(),
                            path: to.clone(),
                        }
                        .into(),
                    );
                }

                Some(thot::Script::Created(to.clone()).into())
            }

            _ => todo!(),
        }
    }

    fn handle_file_renamed(&self, from: &PathBuf, to: &PathBuf) -> Option<thot::Event> {
        if let Some(asset) = self.store.get_path_asset_id(&from).cloned() {
            return Some(
                thot::Asset::Moved {
                    asset,
                    path: to.clone(),
                }
                .into(),
            );
        };

        let project = self.project_by_resource_path(&from).unwrap();
        if let Some(analysis_root) = project.analysis_root_path().as_ref() {
            let script_path = from.strip_prefix(analysis_root).unwrap();
            let scripts = self.store.get_project_scripts(&project.rid).unwrap();
            for script in scripts.values() {
                if script.path.as_path() == script_path {
                    return Some(
                        thot::Script::Moved {
                            script: script.rid.clone(),
                            path: to.clone(),
                        }
                        .into(),
                    );
                }
            }

            if let Ok(script_path) = to.strip_prefix(analysis_root) {
                let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
                let scripts = self.store.get_project_scripts(&project.rid).unwrap();
                if scripts.contains_path(&script_path) {
                    return None;
                }

                let Some(ext) = to.extension() else {
                    return None;
                };

                let ext = ext.to_ascii_lowercase();
                let ext = ext.to_str().unwrap();
                if ScriptLang::supported_extensions().contains(&ext) {
                    return Some(thot::Script::Created(to.clone()).into());
                }

                return None;
            }
        }

        None
    }

    fn handle_folder_created(&self, path: &PathBuf) -> Result<Option<thot::Event>> {
        // ignore thot folder
        if path
            .components()
            .any(|seg| seg.as_os_str() == thot_local::common::thot_dir().as_os_str())
        {
            return Ok(None);
        }

        // ignore graph root
        let project = self.project_by_resource_path(path)?;
        if project.data_root.is_some() {
            let path = fs::canonicalize(&path).unwrap();
            if path == project.data_root_path().unwrap() {
                return Ok(None);
            }
        };

        // ignore if registered container
        if self
            .store
            .get_path_container_canonical(&path)
            .unwrap()
            .is_some()
        {
            return Ok(None);
        }

        // handle if unregistered container
        match ContainerTreeIncrementalLoader::load(path) {
            Ok(graph) => {
                let Some(loaded_container) = self.store.get_container(graph.root()) else {
                    return Ok(Some(thot::Graph::Inserted(graph).into()));
                };

                if loaded_container.base_path().exists() {
                    Ok(Some(thot::Graph::Copied(graph).into()))
                } else {
                    Ok(Some(
                        thot::Graph::Moved {
                            root: graph.root().clone(),
                            path: path.clone(),
                        }
                        .into(),
                    ))
                }
            }

            Err(PartialLoad { errors, graph }) => match errors.get(path) {
                Some(ContainerTreeLoaderError::Dir(err)) if err == &io::ErrorKind::NotFound => {
                    return Ok(Some(thot::Folder::Created(path.clone()).into()));
                }

                Some(ContainerTreeLoaderError::Load(ContainerLoaderError::NotResource)) => {
                    return Ok(Some(thot::Folder::Created(path.clone()).into()));
                }

                _ => {
                    let graph = graph.map(|graph| ContainerTreeTransformer::local_to_core(&graph));
                    return Err(Error::LoadPartial { errors, graph });
                }
            },
        }
    }

    fn handle_folder_removed(&self, path: &PathBuf) -> Option<thot::Event> {
        let Some(container) = self.store.get_path_container(path).cloned() else {
            return None;
        };

        Some(thot::Graph::Removed(container).into())
    }

    fn handle_folder_moved(&self, from: &PathBuf, to: &PathBuf) -> Option<thot::Graph> {
        let Some(root) = self.store.get_path_container(from).cloned() else {
            return None;
        };

        Some(thot::Graph::Moved {
            root,
            path: to.clone(),
        })
    }

    fn handle_folder_renamed(&self, from: &PathBuf, to: &PathBuf) -> Option<thot::Graph> {
        let Some(container) = self.store.get_path_container(from).cloned() else {
            return None;
        };

        Some(thot::Graph::Moved {
            root: container,
            path: to.clone(),
        })
    }

    fn handle_any_created(&self, path: &PathBuf) -> Result<Option<thot::Event>> {
        if path.is_file() {
            self.handle_file_created(path)
        } else if path.is_dir() {
            self.handle_folder_created(path)
        } else {
            Ok(None)
        }
    }

    fn handle_any_removed(&self, path: &PathBuf) -> Option<thot::Event> {
        if let Some(container) = self.store.get_path_container(&path).cloned() {
            return Some(thot::Graph::Removed(container).into());
        }

        if let Some(asset) = self.store.get_path_asset_id(&path).cloned() {
            return Some(thot::Asset::Removed(asset).into());
        }

        let project = self.project_by_resource_path(&path).unwrap();
        let scripts = self.store.get_project_scripts(&project.rid).unwrap();
        if let Ok(script_path) = path.strip_prefix(project.analysis_root_path().unwrap()) {
            let script_path = ResourcePath::new(script_path.to_path_buf()).unwrap();
            if let Some(script) = scripts.by_path(&script_path) {
                return Some(thot::Script::Removed(script.rid.clone()).into());
            }
        }

        None
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
            return Err(Error::DatabaseError("Project not loaded".into()));
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
            return Err(CoreError::ProjectError(CoreProjectError::misconfigured(
                "data root not set",
            ))
            .into());
        };

        if self.store.is_project_graph_loaded(&project.rid) {
            return Ok(());
        }

        let path = project.base_path().join(data_root);
        let graph: ContainerTree = ContainerTreeLoader::load(&path)?;
        self.store.insert_project_graph(project.rid.clone(), graph);

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
