//! Database for storing resources.
use crate::error::Result;
use settings_manager::LocalSettings;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::{Asset as CoreAsset, Script as CoreScript};
use thot_core::types::{ResourceId, ResourceMap};
use thot_local::project::resources::{
    Container as LocalContainer, Project as LocalProject, Scripts as ProjectScripts,
};
use thot_local::types::resource_store::ResourceWrapper;
use thot_local::types::ResourceValue;

// *************
// *** Types ***
// *************

pub type ContainerWrapper = ResourceWrapper<LocalContainer>;

pub type IdMap = HashMap<ResourceId, ResourceId>;

/// Map of [`PathBuf`] to the corresponding [`ResourceId`].
pub type PathMap = HashMap<PathBuf, ResourceId>;

/// Map of [`ResourceId`] to [`Project`](LocalProject).
pub type ProjectMap = ResourceMap<LocalProject>;

/// Map of [`ResourceId`] to [`Container`](LocalContainer).
pub type ContainerMap = ResourceMap<ContainerWrapper>;

/// Project's scripts.
pub type ProjectScriptsMap = HashMap<ResourceId, ProjectScripts>;

// *****************
// *** Datastore ***
// *****************

// @todo: Datastore should only store data.
// Move functionality into Database.
/// A store for [`Container`](LocalContainer)s.
/// Assets can be referenced as well.
///
/// # Notes
/// + Because local Thot resources can only be controlled by a single process
/// a `Datastore` operates as a single repository for all processes requiring access
/// to these resources.
/// This means that the `Datastore` should control all resources it stores.
pub struct Datastore {
    /// [`Project`](LocalProject) store.
    projects: ProjectMap,

    /// Map from a [`Project`](LocalProject)'s path to its [`ResourceId`].
    project_paths: PathMap,

    /// [`Container`](LocalContainer) store.
    containers: ContainerMap,

    /// Map from a [`Container`](LocalContainer)'s path to its [`ResourceId`].
    container_paths: PathMap,

    /// Map from an [`Asset`]'s [`ResourceId`] to its [`Container`]'s.
    assets: IdMap,

    /// Map from a `Script`'s [`ResourceId`] to its `Project`.
    script_projects: IdMap,

    /// Holds a `Project`'s `Scripts`.
    scripts: ProjectScriptsMap,
}

impl Datastore {
    pub fn new() -> Self {
        Datastore {
            projects: ProjectMap::new(),
            project_paths: PathMap::new(),
            containers: ContainerMap::new(),
            container_paths: PathMap::new(),
            assets: IdMap::new(),
            script_projects: IdMap::new(),
            scripts: ProjectScriptsMap::new(),
        }
    }

    // ***************
    // *** project ***
    // ***************

    // @todo: Ensure the `Project` controls the settings file.
    /// Inserts a [`Project`](LocalProject) into the database.
    ///
    /// # Returns
    /// Reference to the inserted `Project`(LocalProject).
    ///
    /// # Panics
    /// + If `project.path()` returns an error.
    pub fn insert_project(&mut self, project: LocalProject) -> Result {
        let cid = project.rid.clone();
        let base_path = project.base_path().expect("invalid `Project` base path");

        self.projects.insert(cid.clone(), project);
        self.project_paths.insert(base_path, cid);

        Ok(())
    }

    /// Gets a [`Project`](LocalProject) from the database if it exists,
    /// otherwise `None`.
    pub fn get_project(&self, rid: &ResourceId) -> Option<&LocalProject> {
        self.projects.get(rid)
    }

    pub fn get_project_mut(&mut self, rid: &ResourceId) -> Option<&mut LocalProject> {
        self.projects.get_mut(&rid)
    }

    pub fn get_path_project(&self, path: &PathBuf) -> Option<&ResourceId> {
        self.project_paths.get(path)
    }

    // *****************
    // *** container ***
    // *****************

    // @todo: Ensure the `Container` controls the settings file.
    /// Inserts a [`Container`](LocalContainer) into the database.
    /// Creates mappings for the [`Container`](LocalContainer)'s [`Assets`].
    ///
    /// # Returns
    /// Old [`Container`](LocalContainer) or `None`.
    ///
    /// # Panics
    /// + If `container.path()` returns an error.
    pub fn insert_container(
        &mut self,
        container: LocalContainer,
    ) -> Result<Option<ContainerWrapper>> {
        let cid = container.rid.clone();
        let base_path = container.base_path()?;

        for (rid, _asset) in container.assets.iter() {
            self.assets.insert(rid.clone(), cid.clone());
        }

        let o_container = self
            .containers
            .insert(cid.clone(), Arc::new(Mutex::new(container)));

        self.container_paths.insert(base_path, cid);
        Ok(o_container)
    }

    // @todo: Ensure the `Container` controls the settings file.
    /// Inserts a [`Container`](LocalContainer) into the database,
    /// recursing on its children to insret the entire tree.
    /// Creates mappings for the [`Container`](LocalContainer)'s [`Assets`].
    pub fn insert_container_tree(&mut self, container: ContainerWrapper) -> Result {
        let (cid, base_path) = {
            let container = container.lock().expect("could not lock `Container`");

            // recurse on children
            for child in container.children.values().clone() {
                let ResourceValue::Resource(child) = child.clone() else {
                    // @todo: Handle `ResourceValue::Path` variant.
                    panic!("child `Container` not loaded");
                };

                self.insert_container_tree(child)?;
            }

            // insest assets
            for (rid, _asset) in container.assets.iter() {
                self.assets.insert(rid.clone(), container.rid.clone());
            }

            // get container info while unwrapped
            (container.rid.clone(), container.base_path()?)
        };

        // path map
        self.container_paths.insert(base_path, cid.clone());

        // insert self
        self.containers.insert(cid.clone(), container);
        Ok(())
    }

    /// Gets a [`Container`](LocalContainer) from the database if it exists,
    /// otherwise `None`.
    pub fn get_container(&self, rid: &ResourceId) -> Option<ContainerWrapper> {
        let Some(container) = self.containers.get(rid) else {
            return None;
        };

        Some(container.clone())
    }

    pub fn get_path_container(&self, path: &Path) -> Option<&ResourceId> {
        self.container_paths.get(path)
    }

    /// Removes a [`Container`](LocalContainer) from the database.
    pub fn remove_container(&mut self, rid: &ResourceId) -> Option<ContainerWrapper> {
        self.containers.remove(rid)
    }

    // *************
    // *** asset ***
    // *************

    /// Gets an [`Asset`](LocalAsset)'s [`Container`](LocalContainer) [`ResourceId`]
    /// from the database if it exists, otherwise `None`.
    pub fn get_asset_container_id(&self, rid: &ResourceId) -> Option<&ResourceId> {
        self.assets.get(rid)
    }

    /// Gets an [`Asset`](LocalAsset)'s [`Container`](LocalContainer)
    /// from the database if it exists, otherwise `None`.
    pub fn get_asset_container(&self, rid: &ResourceId) -> Option<ContainerWrapper> {
        let Some(container) = self.assets.get(rid) else {
            return None;
        };

        self.containers.get(container).cloned()
    }

    /// Inserts a map from the `Asset` to its `Container`.
    pub fn insert_asset(&mut self, asset: ResourceId, container: ResourceId) -> Option<ResourceId> {
        self.assets.insert(asset, container)
    }

    // **************
    // *** script ***
    // **************

    pub fn insert_project_scripts(
        &mut self,
        project: ResourceId,
        scripts: ProjectScripts,
    ) -> Option<ProjectScripts> {
        self.scripts.insert(project, scripts)
    }

    pub fn get_project_scripts(&self, project: &ResourceId) -> Option<&ProjectScripts> {
        self.scripts.get(&project)
    }

    pub fn get_script_project(&self, script: &ResourceId) -> Option<&ResourceId> {
        self.script_projects.get(&script)
    }

    pub fn insert_script(
        &mut self,
        project: ResourceId,
        script: CoreScript,
    ) -> Result<Option<CoreScript>> {
        let Some(scripts) = self.scripts.get_mut(&project) else {
            // project does not exist
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` does not exist".to_string())).into());
        };

        let sid = script.rid.clone();
        let o_script = scripts.insert(sid.clone(), script);
        scripts.save()?;

        // map script
        self.script_projects.insert(project, sid);

        Ok(o_script)
    }

    pub fn get_script(&self, script: &ResourceId) -> Option<&CoreScript> {
        let Some(project) = self.script_projects.get(&script) else {
            return None;
        };

        let Some(scripts) = self.scripts.get(&project) else {
            return None;
        };

        scripts.get(&script)
    }

    pub fn get_script_mut(&mut self, script: ResourceId) -> Option<&mut CoreScript> {
        let Some(project) = self.script_projects.get(&script) else {
            return None;
        };

        let Some(scripts) = self.scripts.get_mut(&project) else {
            return None;
        };

        scripts.get_mut(&script)
    }
}

#[cfg(test)]
#[path = "./store_test.rs"]
mod store_test;
