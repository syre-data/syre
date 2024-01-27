//! High level functionality related to Containers.
use super::project;
use super::resources::Container;
use crate::common::{container_file_of, thot_dir};
use crate::loader::container::Loader as ContainerLoader;
use crate::Result;
use std::path::{self, Path, PathBuf};
use std::{fs, io};
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::project::{Asset, ContainerProperties};
use thot_core::types::ResourceId;

// ***************
// *** Builder ***
// ***************

#[derive(Default)]
pub struct InitNew {
    properties: Option<ContainerProperties>,
}

impl InitNew {
    pub fn properties(&self) -> Option<&ContainerProperties> {
        self.properties.as_ref()
    }

    pub fn set_properties(&mut self, properties: ContainerProperties) {
        self.properties = Some(properties);
    }

    pub fn unset_properties(&mut self) {
        self.properties = None;
    }
}

#[derive(Default)]
pub struct InitExisting {
    recursive: bool,

    /// glob patterns to ignore.
    ignore: Vec<String>,
}

impl InitExisting {
    pub fn set_recursive(&mut self, recursive: bool) {
        self.recursive = recursive;
    }

    pub fn ignored(&self) -> &Vec<String> {
        &self.ignore
    }

    pub fn ignore(&mut self, pattern: impl Into<String>) {
        self.ignore.push(pattern.into());
    }
}

#[derive(Default)]
pub struct InitOptions<I> {
    init: I,
    init_assets: bool,
}

impl<I> InitOptions<I> {
    /// Initialize files as `Asset`s.
    pub fn with_assets(&mut self) {
        self.init_assets = true;
    }

    /// Do not initialize files as `Asset`s.
    pub fn without_assets(&mut self) {
        self.init_assets = false;
    }
}

impl InitOptions<InitNew> {
    /// Create a new folder as a `Container``.
    pub fn new() -> Self {
        InitOptions::default()
    }

    /// Use the given properties to initialize the `Container`.
    ///
    /// # Notes
    /// + `name` is ignored and will be replaced by the folder's name.
    pub fn properties(&mut self, properties: ContainerProperties) {
        self.init.set_properties(properties);
    }

    /// Clears proeprties to set from.
    pub fn unset_properties(&mut self) {
        self.init.unset_properties();
    }

    /// Run the intialization.
    ///
    /// # Returns
    /// [`ResourceId`] of the [`Container`](CoreContainer).
    pub fn build(&self, path: impl AsRef<Path>) -> Result<ResourceId> {
        let path = path.as_ref();
        if path.exists() && project::path_is_resource(path) {
            return Err(CoreError::ResourceError(ResourceError::already_exists(
                "path is already a resource",
            ))
            .into());
        }

        let mut container = Container::new(path);
        if let Some(properties) = self.init.properties() {
            container.properties = properties.clone();
        }

        container.properties.name = container
            .base_path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if self.init_assets {
            for entry in fs::read_dir(container.base_path()).unwrap() {
                let entry = entry.unwrap();
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let asset = Asset::new(entry_path);
                    container.insert_asset(asset);
                }
            }
        }

        container.save()?;
        Ok(container.rid.clone())
    }
}

impl InitOptions<InitExisting> {
    /// Initialize an existing folder or folder tree as a `Container`.
    pub fn init() -> Self {
        InitOptions::default()
    }

    /// Set whether to recurse into subfolders.
    pub fn recurse(&mut self, recursive: bool) {
        self.init.set_recursive(recursive);
    }

    /// Ignore a path and it's subfolders when recursing.
    /// Ignored if `recurse` is `false`.
    ///
    /// # Arguments
    /// + `pattern`: A glob pattern to ignore, relative to the
    /// `Container` graph root.
    pub fn ignore(&mut self, pattern: impl Into<String>) {
        self.init.ignore(pattern);
    }

    /// Intialize the path as a `Container` tree.
    ///
    /// # Returns
    /// [`ResourceId`] of the root [`Container`](CoreContainer).
    ///
    /// # Errors
    /// + If a path is already a resource but can not be loaded as a Container.
    ///
    /// # Notes
    /// + If `path` is already initialized as a `Container` it is re-initialized,
    /// with all properties being maintained, but `Asset`s being updated.
    ///  - If `recurse` is `true`, this applies for folders within the subtree, too.
    ///
    /// + `Container` name will be updated to match the folder.
    /// + Hidden files (i.e. Files whose name starts with a period (.)) are ignored as `Asset`s.
    pub fn build(&self, path: impl AsRef<Path>) -> Result<ResourceId> {
        /// Initialize a path as a Container.
        /// Used to recurse.
        ///
        /// # Arguments
        /// + `ignore`: Absolute paths to ignore.
        ///     Ignored if `recurse` is `false`.
        ///
        /// # Notes
        /// + Hidden files are ignored as `Asset`s.
        fn init_container(
            path: impl AsRef<Path>,
            init_assets: bool,
            recurse: bool,
            ignore: &Vec<PathBuf>,
        ) -> Result<ResourceId> {
            let path = path.as_ref();
            // TODO What if path is a project?
            let mut container = if path_is_container(path) {
                ContainerLoader::load(path)?
            } else {
                Container::new(path)
            };

            container.properties.name = container
                .base_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let mut dirs = Vec::new();
            let mut files = Vec::new();
            for entry in fs::read_dir(container.base_path()).unwrap() {
                let entry = entry.unwrap();
                let entry_path = entry.path();
                if entry_path.is_file() {
                    files.push(entry_path);
                } else if entry_path.is_dir() {
                    if entry_path.components().any(|seg| match seg {
                        path::Component::Normal(seg) => seg == thot_dir(),
                        _ => false,
                    }) {
                        continue;
                    }

                    dirs.push(entry_path);
                }
            }

            let container_path = fs::canonicalize(container.base_path()).unwrap();
            let asset_paths = container
                .assets
                .values()
                .map(|asset| {
                    let asset_path = container.base_path().join(asset.path.as_path());
                    fs::canonicalize(asset_path).unwrap()
                })
                .collect::<Vec<_>>();

            if init_assets {
                for file_path in files {
                    let file_path = fs::canonicalize(file_path).unwrap();
                    if asset_paths.contains(&file_path) {
                        continue;
                    }

                    // ignore hidden files as assets
                    if let Some(file_name) = file_path.file_name() {
                        if let Some(file_name) = file_name.to_str() {
                            if file_name.starts_with(".") {
                                continue;
                            }
                        }
                    }

                    let rel_path = file_path
                        .strip_prefix(&container_path)
                        .unwrap()
                        .to_path_buf();

                    let asset = Asset::new(rel_path);
                    container.insert_asset(asset);
                }
            }
            container.save()?;

            if recurse {
                for dir_path in dirs.into_iter().filter(|path| !ignore.contains(path)) {
                    init_container(dir_path, init_assets, recurse, ignore)?;
                }
            }

            Ok(container.rid.clone())
        }

        // main
        let path = path.as_ref();
        if !path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotADirectory, "path does not exist").into());
        }

        let ignore = self
            .init
            .ignored()
            .iter()
            .map(|pattern| {
                let pattern = path.join(pattern).to_str().unwrap().to_string();
                let mut match_options = glob::MatchOptions::new();
                match_options.case_sensitive = false;

                glob::glob_with(&pattern, match_options)
                    .unwrap()
                    .map(|path| PathBuf::from(path.unwrap()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        init_container(path, self.init_assets, self.init.recursive, &ignore)
    }
}

// *****************
// *** Functions ***
// *****************

/// Convenience function to create a new folder as a `Container`.
///
/// Equivalent to
/// ```
/// let builder = InitOptions::new();
/// builder.build(path)?;
/// ```
pub fn new(path: impl AsRef<Path>) -> Result<ResourceId> {
    let builder = InitOptions::new();
    builder.build(path.as_ref())
}

/// Move a Container, including all its resources (children, assets, settings,
/// etc.) a to a new location.
pub fn mv(rid: ResourceId, to: impl AsRef<Path>) -> Result {
    todo!();
}

/// Remove a folder as a Container without deleting it.
///
/// # See also
/// + `delete'
pub fn remove(rid: ResourceId) -> Result {
    todo!();
}

/// Delete a Container and all its resources (children, assets, settings, etc.).
///
/// # Notes
/// This will permananently delete all the associated resources from the file system.
/// For a non-destructive way to remove a Container and its resource see `remove`.
///
/// # See also
/// + `remove`
pub fn delete(rid: ResourceId) -> Result {
    todo!();
}

/// Updates the Container with the matching resource id.
pub fn update(container: Container) -> Result {
    todo!();
}

/// Returns whether or not the path is a Container.
/// Checks if <path>/<THOT_DIR>/<CONTAINER_FILE> exists.
pub fn path_is_container(path: &Path) -> bool {
    let c_path = container_file_of(path);
    c_path.exists()
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
