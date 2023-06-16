//! Database for storing resources.
use crate::error::Result;
use has_id::HasId;
use settings_manager::Settings;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thot_core::db::{SearchFilter, StandardSearchFilter as StdFilter};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::graph::ResourceTree;
use thot_core::project::{Asset, Container as CoreContainer, Metadata, Script as CoreScript};
use thot_core::types::{ResourceId, ResourceMap, ResourcePath};
use thot_local::project::resources::{
    Container as LocalContainer, Project as LocalProject, Scripts as ProjectScripts,
};

// *************
// *** Types ***
// *************

// TODO[l]: Types don't need to be `pub`.
pub struct PathMap<T>(HashMap<PathBuf, T>);
impl<T> PathMap<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, key: &Path) -> Option<&T> {
        let key = fs::canonicalize(&key).unwrap();
        self.0.get(&key)
    }

    pub fn get_mut(&mut self, key: &Path) -> Option<&mut T> {
        let key = fs::canonicalize(key).unwrap();
        self.0.get_mut(&key)
    }

    pub fn insert(&mut self, key: PathBuf, value: T) -> Option<T> {
        let key = fs::canonicalize(key).unwrap();
        self.0.insert(key, value)
    }

    pub fn remove(&mut self, key: &Path) -> Option<T> {
        let key = fs::canonicalize(key).unwrap();
        self.0.remove(&key)
    }
}

pub type ContainerTree = ResourceTree<LocalContainer>;

pub type IdMap = HashMap<ResourceId, ResourceId>;

/// Map of [`ResourceId`] to [`Project`](LocalProject).
pub type ProjectMap = ResourceMap<LocalProject>;

/// Map from [`Project`](LocalProject)s to their [`Script`](CoreScript)s.
pub type ProjectScriptsMap = HashMap<ResourceId, ProjectScripts>;

/// Map from a `Project`'s id to its [`ContainerTree`]
pub type ProjectGraphMap = HashMap<ResourceId, ContainerTree>;

// *****************
// *** Datastore ***
// *****************

// @todo: Paths should always be canonicalized.

/// A store for [`Container`](LocalContainer)s.
/// Assets can be referenced as well.
///
///
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
    project_paths: PathMap<ResourceId>,

    /// Map from [`Project`] to its graph.
    graphs: ProjectGraphMap,

    /// Map from a [`Container`](LocalContainer)'s path to its [`ResourceId`].
    container_paths: PathMap<ResourceId>,

    /// Map from a `Container`'s id to its `Project`'s.
    container_projects: IdMap,

    /// Map from an [`Asset`]'s [`ResourceId`] to its [`Container`]'s.
    assets: IdMap,

    /// Map from a [`Script`](CoreScript)'s [`ResourceId`] to its `Project`.
    script_projects: IdMap,

    /// Holds a `Project`'s `Scripts`.
    scripts: ProjectScriptsMap,
}

impl Datastore {
    pub fn new() -> Self {
        Datastore {
            projects: ProjectMap::new(),
            project_paths: PathMap::new(),
            graphs: ProjectGraphMap::new(),
            container_paths: PathMap::new(),
            container_projects: IdMap::new(),
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
        let pid = project.rid.clone();
        let project_path = project.base_path().to_path_buf();

        self.projects.insert(pid.clone(), project);
        self.project_paths.insert(project_path, pid);

        Ok(())
    }

    /// Gets a [`Project`](LocalProject) from the database if it exists,
    /// otherwise `None`.
    pub fn get_project(&self, rid: &ResourceId) -> Option<&LocalProject> {
        self.projects.get(rid)
    }

    /// Gets a `mut`able [`Project`](LocalProject) from the database if it exists,
    /// otherwise `None`.
    pub fn get_project_mut(&mut self, rid: &ResourceId) -> Option<&mut LocalProject> {
        self.projects.get_mut(rid)
    }

    /// Gets the `Project` associated to the given path.
    pub fn get_path_project(&self, path: &Path) -> Option<&ResourceId> {
        self.project_paths.get(path)
    }

    /// Gets the `Project` the `Container` belongs to.
    pub fn get_container_project(&self, container: &ResourceId) -> Option<&ResourceId> {
        self.container_projects.get(container)
    }

    // *************
    // *** graph ***
    // *************

    /// Gets a [`Project`](LocalProjet)'s [`ContainerTree`].
    ///
    /// # Arguments
    /// 1. [`ResourceId`] of the [`Project`](LocalProjet).
    pub fn get_project_graph(&self, rid: &ResourceId) -> Option<&ContainerTree> {
        self.graphs.get(&rid)
    }

    /// Gets the graph of a `Container`.
    ///
    /// # Arguments
    /// 1. [`ResourceId`] of the [`Container`](LocalContainer).
    pub fn get_container_graph(&self, container: &ResourceId) -> Option<&ContainerTree> {
        let Some(project) = self.container_projects.get(&container) else {
            return None;
        };

        let graph = self
            .graphs
            .get(project)
            .expect("`Project` present without graph");

        Some(graph)
    }

    /// Gets the `mut`able graph of a `Container`.
    ///
    /// # Arguments
    /// 1. [`ResourceId`] of the [`Container`](LocalContainer).
    fn get_container_graph_mut(&mut self, container: &ResourceId) -> Option<&mut ContainerTree> {
        let Some(project) = self.container_projects.get(&container) else {
            return None;
        };

        let graph = self
            .graphs
            .get_mut(project)
            .expect("`Project` present without graph");

        Some(graph)
    }

    // TODO: DRY `insert_project_graph` and `insert_sub_graph`.
    /// Inserts a [`Project`](LocalProjet)'s [`ContainerTree`].
    ///
    /// # Arguments
    /// 1. [`ResourceId`] of the [`Project`](LocalProjet).
    /// 2. The [`ContainerTree`].
    ///
    /// # Returns
    /// The old [`ContainerTree`].
    pub fn insert_project_graph(
        &mut self,
        rid: ResourceId,
        graph: ContainerTree,
    ) -> Option<ContainerTree> {
        // map containers
        for (cid, node) in graph.nodes().iter() {
            self.container_projects.insert(cid.clone(), rid.clone());
            self.container_paths
                .insert(node.base_path().into(), cid.clone());

            // map assets to containers
            for aid in node.data().assets.keys() {
                self.assets.insert(aid.clone(), cid.clone());
            }
        }

        self.graphs.insert(rid, graph)
    }

    /// Insert a graph into another.
    pub fn insert_subgraph(
        &mut self,
        parent: &ResourceId,
        graph: ResourceTree<LocalContainer>,
    ) -> Result {
        let Some(project) = self.get_container_project(parent).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` `Project` not found")).into());
        };

        let p_graph = self
            .graphs
            .get_mut(&project)
            .expect("`Project` graph not found");

        for (cid, container) in graph.nodes() {
            // map container to project
            self.container_projects.insert(cid.clone(), project.clone());

            // map path to container
            self.container_paths
                .insert(container.base_path().into(), cid.clone());

            // map assets to containers
            for aid in container.assets.keys() {
                self.assets.insert(aid.clone(), cid.clone());
            }
        }

        p_graph.insert_tree(parent, graph)?;
        Ok(())
    }

    /// Remove the subgraph with the given root.
    ///
    /// # Returns
    /// Removed subgraph root.
    #[tracing::instrument(skip(self))]
    pub fn remove_subgraph(&mut self, root: &ResourceId) -> Result<ContainerTree> {
        let Some(project) = self.get_container_project(root).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` `Project` not found")).into());
        };

        let p_graph = self
            .graphs
            .get_mut(&project)
            .expect("`Project` graph not found");

        let sub_graph = p_graph.remove(root)?;
        for (cid, container) in sub_graph.nodes() {
            // map container to project
            self.container_projects.remove(cid);

            // map path to container
            self.container_paths.remove(&container.base_path());

            // map assets to containers
            for aid in container.assets.keys() {
                self.assets.remove(aid);
            }
        }

        Ok(sub_graph)
    }

    // *****************
    // *** container ***
    // *****************

    /// Gets a [`Container`](LocalContainer).
    pub fn get_container(&self, container: &ResourceId) -> Option<&LocalContainer> {
        let Some(graph) = self.get_container_graph(container) else {
            return None;
        };

        let Some(node) = graph.get(container) else {
            return None;
        };

        Some(node.data())
    }

    /// Gets a `mut`able [`Container`](LocalContainer).
    pub fn get_container_mut(&mut self, container: &ResourceId) -> Option<&mut LocalContainer> {
        let Some(graph) = self.get_container_graph_mut(container) else {
            return None;
        };

        let Some(node) = graph.get_mut(container) else {
            return None;
        };

        Some(node)
    }

    /// Get a Container with inherited metadata.
    pub fn get_container_with_metadata(&self, container: &ResourceId) -> Option<CoreContainer> {
        let Some(container) = self.get_container(container) else {
            return None;
        };

        let graph = self
            .get_container_graph(container.id())
            .expect("could not find `Container`'s graph");

        let metadata = graph.ancestors(container.id()).into_iter().rfold(
            Metadata::new(),
            |mut metadata, ancestor| {
                let container = graph.get(&ancestor).expect("`Container` not found");
                for (key, value) in container.properties.metadata.clone() {
                    metadata.insert(key, value);
                }

                metadata
            },
        );

        let mut container = (*container).clone();
        container.properties.metadata = metadata;
        Some(container)
    }

    /// Finds `Container`'s that match the filter.
    ///
    /// # Arguments
    /// 1. Root of subtree to search within.
    /// 2. Filter.
    ///
    /// # Note
    /// + `Metadata` is not inherited.
    ///
    /// # See also
    /// + [`find_containers_with_metadata`]
    pub fn find_containers(
        &self,
        root: &ResourceId,
        filter: StdFilter,
    ) -> HashSet<&LocalContainer> {
        let mut found = HashSet::new();
        let Some(graph) = self.get_container_graph(root) else {
            return found;
        };

        let nodes = graph
            .descendants(&root)
            .expect("`Container` not found in graph");

        for node in nodes {
            let node = graph.get(&node).expect("`Container` not found in graph");

            // @todo[4]: Implement for `LocalContainer`.
            let container: CoreContainer = (*node.data()).clone();
            if filter.matches(&container) {
                found.insert(node.data());
            }
        }

        found
    }

    // TODO[h] Assets should have paths canonicalized and made absolute.
    /// Finds `Container`'s that match the filter with inherited `Metadata`.
    ///
    /// # Arguments
    /// 1. Root of subtree to search within.
    /// 2. Filter.
    ///
    /// # See also
    /// + [`find_containers`]
    #[tracing::instrument(skip(self))]
    pub fn find_containers_with_metadata(
        &self,
        root: &ResourceId,
        filter: StdFilter,
    ) -> HashSet<CoreContainer> {
        /// Recursively finds mathcing `Containers`, inheriting metadata.
        #[tracing::instrument(skip(graph))]
        fn find_containers_with_metadata_recursive(
            root: &ResourceId,
            graph: &ContainerTree,
            filter: StdFilter,
            mut metadata: Metadata,
        ) -> HashSet<CoreContainer> {
            let mut found = HashSet::new();
            let root = graph.get(root).expect("`Container` not in graph");

            let children = graph.children(root.id()).expect("`Container` not in graph");
            for (key, value) in root.data().properties.metadata.clone().into_iter() {
                metadata.insert(key, value);
            }

            for child in children {
                let node = graph.get(&child).expect("child `Container` not in graph");
                for matching in find_containers_with_metadata_recursive(
                    node.id(),
                    &graph,
                    filter.clone(),
                    metadata.clone(),
                )
                .into_iter()
                {
                    found.insert(matching);
                }
            }

            let mut container: CoreContainer = (*root.data()).clone();
            container.properties.metadata = metadata;
            if filter.matches(&container) {
                found.insert(container);
            }

            found
        }

        // run fn
        let Some(graph) = self.get_container_graph(root) else {
            return HashSet::new();
        };

        let metadata =
            graph
                .ancestors(root)
                .into_iter()
                .rfold(Metadata::new(), |mut metadata, ancestor| {
                    let container = graph.get(&ancestor).expect("`Container` not found");
                    for (key, value) in container.properties.metadata.clone() {
                        metadata.insert(key, value);
                    }

                    metadata
                });

        find_containers_with_metadata_recursive(root, graph, filter, metadata)
    }

    /// Gets a `Container`'s id by path.
    pub fn get_path_container(&self, path: &Path) -> Option<&ResourceId> {
        self.container_paths.get(path)
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
    pub fn get_asset_container(&self, rid: &ResourceId) -> Option<&LocalContainer> {
        let Some(container) = self.assets.get(rid) else {
            return None;
        };

        let container = self
            .get_container(container)
            .expect("`Container` not found in graph");

        Some(container)
    }

    /// Inserts a map from the `Asset` to its `Container`.
    pub fn insert_asset(&mut self, asset: ResourceId, container: ResourceId) -> Option<ResourceId> {
        self.assets.insert(asset, container)
    }

    /// Adds an [`Asset`](CoreAsset) to a `Container`.
    pub fn add_asset(&mut self, mut asset: Asset, container: ResourceId) -> Result<Option<Asset>> {
        let Some(project) = self.container_projects.get(&container) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` is not loaded")).into());
        };

        let graph = self
            .graphs
            .get_mut(project)
            .expect("`Project` present without graph");

        let Some(container) = graph.get_mut(&container) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` is not loaded")).into());
        };

        // check if asset with same path already extists
        for c_asset in container.assets.values() {
            if asset.path == c_asset.path {
                asset.rid = c_asset.rid.clone();
                break;
            }
        }

        let aid = asset.rid.clone();
        let cid = container.rid.clone();
        let o_asset = container.assets.insert(aid.clone(), asset);
        container.save()?;

        self.insert_asset(aid, cid);
        Ok(o_asset)
    }

    /// Removes an `Asset` from its `Container`.
    /// Deletes the related file if it exists.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_asset(&mut self, rid: &ResourceId) -> Result<Option<Asset>> {
        let Some(cid) = self.assets.get(rid).cloned() else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` is not loaded")).into());
        };

        let Some(graph) = self.get_container_graph_mut(&cid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container`'s graph is not loaded")).into());
        };

        let Some(container) = graph.get_mut(&cid) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Container` not in graph")).into());
        };

        let container_path = container.base_path().to_path_buf();
        let asset = container.assets.remove(rid);
        container.save()?;

        self.assets.remove(rid);
        if let Some(asset) = asset.as_ref() {
            let path = container_path.join(asset.path.as_path());
            trash::delete(path)?;
        };

        Ok(asset)
    }

    /// Finds `Asset`'s that match the filter.
    ///
    /// # Arguments
    /// 1. Root of subtree to search within.
    /// 2. Filter.
    ///
    /// # Note
    /// + `Metadata` is not inherited.
    ///
    /// # See also
    /// + [`find_assets_with_metadata`]
    pub fn find_assets(&self, root: &ResourceId, filter: StdFilter) -> HashSet<Asset> {
        let mut found = HashSet::new();
        let Some(graph) = self.get_container_graph(root) else {
            return found;
        };

        let nodes = graph
            .descendants(root)
            .expect("`Container` not found in graph");

        for node in nodes {
            let container = graph.get(&node).expect("`Container` not found in graph");

            for asset in container.data().assets.values() {
                if filter.matches(asset) {
                    // set path to absolute
                    let mut asset = asset.clone();
                    let path = container.base_path().join(asset.path.as_path());
                    asset.path = ResourcePath::new(path).expect("could not set absolute path");

                    found.insert(asset);
                }
            }
        }

        found
    }

    /// Finds `Asset`'s that match the filter with inherited `Metadata`.
    ///
    /// # Arguments
    /// 1. Root of subtree to search within.
    /// 2. Filter.
    ///
    /// # See also
    /// + [`find_assets`]
    pub fn find_assets_with_metadata(
        &self,
        root: &ResourceId,
        filter: StdFilter,
    ) -> HashSet<Asset> {
        /// Recursively finds mathcing `Containers`, inheriting metadata.
        fn find_assets_with_metadata_recursive(
            root: &ResourceId,
            graph: &ContainerTree,
            filter: StdFilter,
            mut metadata: Metadata,
        ) -> HashSet<Asset> {
            let mut found = HashSet::new();
            let root = graph.get(root).expect("`Container` not in graph");

            let children = graph.children(root.id()).expect("`Container` not in graph");
            for (key, value) in root.data().properties.metadata.clone().into_iter() {
                metadata.insert(key, value);
            }

            for child in children {
                let node = graph.get(&child).expect("child `Container` not in graph");
                for matching in find_assets_with_metadata_recursive(
                    node.id(),
                    &graph,
                    filter.clone(),
                    metadata.clone(),
                )
                .into_iter()
                {
                    found.insert(matching);
                }
            }

            for asset in root.data().assets.values() {
                let mut asset = asset.clone();
                for (key, value) in metadata.clone().into_iter() {
                    asset.properties.metadata.insert(key, value);
                }

                if filter.matches(&asset) {
                    // set path to absolute
                    let mut path = root.base_path().join(asset.path.as_path());
                    asset.path = ResourcePath::new(path).expect("could not set absolute path");

                    found.insert(asset);
                }
            }

            found
        }

        // find mathing containers
        let Some(graph) = self.get_container_graph(root) else {
            return HashSet::new();
        };

        let metadata =
            graph
                .ancestors(root)
                .into_iter()
                .rfold(Metadata::new(), |mut metadata, ancestor| {
                    let container = graph.get(&ancestor).expect("`Container` not found");
                    for (key, value) in container.properties.metadata.clone() {
                        metadata.insert(key, value);
                    }

                    metadata
                });

        find_assets_with_metadata_recursive(root, graph, filter, metadata)
    }

    // **************
    // *** script ***
    // **************

    pub fn insert_project_scripts(
        &mut self,
        project: ResourceId,
        scripts: ProjectScripts,
    ) -> Option<ProjectScripts> {
        // map scripts
        for script in scripts.keys() {
            self.script_projects.insert(script.clone(), project.clone());
        }

        self.scripts.insert(project, scripts)
    }

    /// Gets a `Project`'s `Script`s.
    pub fn get_project_scripts(&self, project: &ResourceId) -> Option<&ProjectScripts> {
        self.scripts.get(&project)
    }

    /// Gets the `Project` of a `Script`.
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
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project` does not exist")).into());
        };

        let sid = script.rid.clone();
        let o_script = scripts.insert(sid.clone(), script);
        scripts.save()?;

        // map script
        self.script_projects.insert(sid, project);

        Ok(o_script)
    }

    /// Remove a `Script` from a `Project`.
    /// Removes all `Container` script associations.
    ///
    /// # Returns
    /// Removed `Script`.
    pub fn remove_project_script(
        &mut self,
        project: &ResourceId,
        script: &ResourceId,
    ) -> Result<Option<CoreScript>> {
        let Some(scripts) = self.scripts.get_mut(&project) else {
            // project does not exist
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project`'s `Scripts` does not exist")).into());
        };

        // remove association from contiainers
        let Some(graph) = self.graphs.get_mut(project) else {
            return Err(CoreError::ResourceError(ResourceError::DoesNotExist("`Project`'s graph does not exist")).into());
        };

        for (_cid, container) in graph.iter_nodes_mut() {
            container.scripts.remove(script);
            container.save()?;
        }

        // remove from project
        let o_script = scripts.remove(script);
        scripts.save()?;

        // remove map script
        self.script_projects.remove(script);

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
