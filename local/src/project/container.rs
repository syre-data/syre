use crate::{
    common,
    error::{Error, Result},
    file_resource::LocalResource,
};
use chrono::{DateTime, Utc};
use has_id::HasId;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    hash::{Hash, Hasher},
    io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    result::Result as StdResult,
};
use syre_core::{
    error::{Error as CoreError, Resource as ResourceError},
    project::{AnalysisAssociation, Asset, Container as CoreContainer, ContainerProperties},
    types::{ResourceId, ResourceMap, UserId, UserPermissions},
};

#[cfg(feature = "fs")]
pub use functions::*;

/// Properties for a Container.
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct StoredProperties {
    pub rid: ResourceId,
    pub properties: ContainerProperties,
    pub analyses: Vec<AnalysisAssociation>,
}

#[cfg(feature = "fs")]
impl StoredProperties {
    /// # Arguments
    /// 1. `base_path`: Base path of the container.
    pub fn save(&self, base_path: impl AsRef<Path>) -> StdResult<(), io::Error> {
        let path = common::container_file_of(base_path);
        fs::create_dir_all(path.parent().expect("invalid Container path"))?;
        fs::write(path, serde_json::to_string_pretty(self).unwrap())?;
        Ok(())
    }
}

impl From<CoreContainer> for StoredProperties {
    fn from(container: CoreContainer) -> Self {
        Self {
            rid: container.rid().clone(),
            properties: container.properties,
            analyses: container.analyses,
        }
    }
}

#[derive(Debug)]
pub struct Container {
    pub(crate) base_path: PathBuf,
    pub inner: CoreContainer,
    pub settings: Settings,
}

impl Container {
    /// Create a new Container at the given base path.
    ///
    /// # Arguments
    /// 1. Path to the Container.
    ///
    /// # Notes
    /// + No changes or checks are made to the file system.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let name = PathBuf::from(path.clone());
        let name = name.file_name().expect("invalid path");
        let name: String = name.to_string_lossy().to_string();

        Self {
            base_path: path,
            inner: CoreContainer::new(name),
            settings: Settings::new(),
        }
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
        self.base_path = path.into();
    }

    pub fn buckets(&self) -> Vec<PathBuf> {
        self.assets
            .iter()
            .filter_map(|asset| asset.bucket())
            .collect()
    }

    /// Returns if the container is already associated with the analysis with the given id,
    /// regardless of the associations priority or autorun status.
    pub fn contains_analysis_association(&self, rid: &ResourceId) -> bool {
        self.analyses
            .iter()
            .any(|association| association.analysis() == rid)
    }

    /// Adds an association to the Container.
    /// Errors if an association with the analysis already exists.
    ///
    /// # See also
    /// + `set_analysis_association`
    pub fn add_analysis_association(&mut self, association: AnalysisAssociation) -> Result {
        if self.contains_analysis_association(association.analysis()) {
            return Err(Error::Core(CoreError::Resource(
                ResourceError::already_exists("Association with analysis already exists"),
            )));
        }

        self.analyses.push(association);
        Ok(())
    }

    /// Sets or adds an analysis association with the Container.
    ///
    /// # See also
    /// + [`add_analysis_association`]
    pub fn set_analysis_association(&mut self, association: AnalysisAssociation) {
        self.analyses
            .retain(|a| a.analysis() != association.analysis());
        self.analyses.push(association);
    }

    /// Removes an association with the given analysis.
    pub fn remove_analysis_association(&mut self, rid: &ResourceId) {
        self.analyses
            .retain(|association| association.analysis() != rid);
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Breaks self into parts.
    ///
    /// # Returns
    /// Tuple of (properties, settings, base path).
    pub fn into_parts(self) -> (CoreContainer, Settings, PathBuf) {
        let Self {
            inner: container,
            base_path,
            settings,
        } = self;

        (container, settings, base_path)
    }
}

#[cfg(feature = "fs")]
impl Container {
    /// Save all data.
    pub fn save(&self) -> StdResult<(), error::Save> {
        let properties_path = <Self as LocalResource<StoredProperties>>::path(self);
        let assets_path = <Self as LocalResource<Vec<Asset>>>::path(self);
        let settings_path = <Self as LocalResource<Settings>>::path(self);

        let app_folder = properties_path.parent().expect("invalid Container path");
        fs::create_dir_all(app_folder).map_err(error::Save::CreateDir)?;

        #[cfg(target_os = "windows")]
        if let Err(err) = common::fs::hide_folder(app_folder) {
            #[cfg(feature = "tracing")]
            tracing::warn!("could not hide folder {app_folder:?}: {err:?}");
        }

        let properties: StoredProperties = self.inner.clone().into();

        let save_properties = fs::write(
            properties_path,
            serde_json::to_string_pretty(&properties).unwrap(),
        );

        let save_assets = fs::write(
            assets_path,
            serde_json::to_string_pretty(&self.assets).unwrap(),
        );

        let save_settings = fs::write(
            settings_path,
            serde_json::to_string_pretty(&self.settings).unwrap(),
        );

        if save_properties.is_err() || save_assets.is_err() || save_settings.is_err() {
            Err(error::Save::SaveFiles {
                properties: save_properties.err(),
                assets: save_assets.err(),
                settings: save_settings.err(),
            })
        } else {
            Ok(())
        }
    }
}

impl PartialEq for Container {
    fn eq(&self, other: &Container) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Container {}

impl Deref for Container {
    type Target = CoreContainer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Container {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Hash for Container {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rid().hash(state);
    }
}

impl HasId for Container {
    type Id = ResourceId;

    fn id(&self) -> &Self::Id {
        &self.inner.id()
    }
}

impl LocalResource<StoredProperties> for Container {
    fn rel_path() -> PathBuf {
        common::container_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

/// Settings for a Container
#[derive(PartialEq, Serialize, Deserialize, Clone, Default, Debug)]
pub struct Settings {
    pub creator: Option<UserId>,
    pub created: DateTime<Utc>,
    pub permissions: ResourceMap<UserPermissions>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            creator: None,
            created: Utc::now(),
            permissions: ResourceMap::new(),
        }
    }
}

/// Container assets.
#[derive(
    derive_more::From,
    derive_more::Deref,
    derive_more::DerefMut,
    Serialize,
    Deserialize,
    Clone,
    Default,
    Debug,
)]
#[serde(transparent)]
pub struct Assets(Vec<Asset>);

impl Assets {
    pub fn into_inner(self) -> Vec<Asset> {
        self.0
    }

    /// Save the container properties.
    ///
    /// # Arguments
    /// 1. `base_path`: Base path of the container the properties represent.
    pub fn save(&self, base_path: impl AsRef<Path>) -> StdResult<(), io::Error> {
        let path = common::assets_file_of(base_path);
        fs::create_dir_all(path.parent().expect("invalid Container path"))?;
        fs::write(path, serde_json::to_string_pretty(self).unwrap())?;
        Ok(())
    }
}

impl LocalResource<Vec<Asset>> for Container {
    fn rel_path() -> PathBuf {
        common::assets_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

impl LocalResource<Settings> for Container {
    fn rel_path() -> PathBuf {
        common::container_settings_file()
    }

    fn base_path(&self) -> &Path {
        &self.base_path
    }
}

pub struct Builder {
    base_path: PathBuf,
    properties: Option<ContainerProperties>,
    analyses: Option<Vec<AnalysisAssociation>>,
    settings: Option<Settings>,
}

impl Builder {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            properties: None,
            analyses: None,
            settings: None,
        }
    }

    pub fn with_properties(&mut self, properties: ContainerProperties) {
        let _ = self.properties.insert(properties);
    }

    pub fn with_analyses(&mut self, associations: Vec<AnalysisAssociation>) {
        let _ = self.analyses.insert(associations);
    }

    pub fn with_settings(&mut self, settings: Settings) {
        let _ = self.settings.insert(settings);
    }

    pub fn build(self) -> Container {
        let Builder {
            base_path,
            properties,
            analyses,
            settings,
        } = self;

        let mut container = Container::new(base_path);
        if let Some(properties) = properties {
            container.inner.properties = properties;
        }

        if let Some(associations) = analyses {
            container.inner.analyses = associations;
        }

        if let Some(settings_src) = settings {
            let Settings {
                creator,
                permissions,
                ..
            } = settings_src;
            let mut settings = Settings::new();
            settings.creator = creator;
            settings.permissions = permissions;
            container.settings = settings;
        }

        container
    }
}

#[cfg(feature = "fs")]
/// High level functionality related to Containers.
pub mod functions {
    use super::{builder, error};
    use crate::common::container_file_of;
    use std::path::Path;
    use syre_core::types::ResourceId;

    /// Convenience function to create a new folder as a `Container`.
    ///
    /// Equivalent to
    /// ```
    /// let builder = InitOptions::new();
    /// builder.build(path)?;
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Result<ResourceId, error::Build> {
        let builder = builder::InitOptions::new();
        builder.build(path.as_ref())
    }

    /// Returns whether or not the path is a Container.
    /// Checks if <path>/<APP_DIR>/<CONTAINER_FILE> exists.
    pub fn path_is_container(path: &Path) -> bool {
        let c_path = container_file_of(path);
        c_path.exists()
    }
}

#[cfg(feature = "fs")]
pub mod builder {
    //! Build containers.
    use super::{super::project, error, Container};
    use crate::{common::app_dir, loader::container::Loader as ContainerLoader};
    use std::{
        fs,
        path::{self, Path, PathBuf},
    };
    use syre_core::{
        project::{Asset, ContainerProperties},
        types::ResourceId,
    };

    /// Initialize a new folder.
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

    /// Initialize an existing folder.
    #[derive(Default)]
    pub struct InitExisting {
        recursive: bool,

        /// Create new ids for resources.
        new_ids: bool,

        /// glob patterns to ignore.
        ignore: Vec<String>,
    }

    impl InitExisting {
        pub fn set_recursive(&mut self, recursive: bool) {
            self.recursive = recursive;
        }

        pub fn with_new_ids(&mut self, new_ids: bool) {
            self.new_ids = new_ids;
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
        /// Create a new folder as a `Container`.
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

        /// Clears properties.
        pub fn unset_properties(&mut self) {
            self.init.unset_properties();
        }

        /// Run the intialization.
        ///
        /// # Returns
        /// [`ResourceId`] of the [`Container`](CoreContainer).
        pub fn build(&self, path: impl AsRef<Path>) -> Result<ResourceId, error::Build> {
            let path = path.as_ref();
            if path.exists() && project::functions::path_is_resource(path) {
                return Err(error::Build::AlreadyResource);
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
                        container.assets.push(asset);
                    }
                }
            }

            if let Err(err) = container.save() {
                let err = match err {
                    error::Save::CreateDir(error) => error,
                    error::Save::SaveFiles {
                        properties,
                        assets,
                        settings,
                    } => {
                        if let Some(err) = properties {
                            err
                        } else if let Some(err) = assets {
                            err
                        } else {
                            settings.unwrap()
                        }
                    }
                };

                return Err(error::Build::Save(err.kind()));
            }

            Ok(container.rid().clone())
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

        /// Set whether new ids should be assigned to resources.
        pub fn with_new_ids(&mut self, new_ids: bool) {
            self.init.with_new_ids(new_ids);
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
        pub fn build(&self, path: impl AsRef<Path>) -> Result<ResourceId, error::Build> {
            /// Initialize a path as a Container.
            /// Used to recurse.
            ///
            /// # Arguments
            /// + `ignore`: Absolute paths to ignore.
            ///     No effect if `recurse` is `false`.
            ///
            /// # Notes
            /// + Hidden files are ignored as `Asset`s.
            fn init_container(
                path: impl Into<PathBuf>,
                init_assets: bool,
                new_ids: bool,
                recurse: bool,
                ignore: &Vec<PathBuf>,
            ) -> Result<ResourceId, error::Build> {
                let path: PathBuf = path.into();
                // TODO: What if path is a project?
                let mut container = if super::path_is_container(&path) {
                    let container_state = match ContainerLoader::load(&path) {
                        Ok(container) => container,
                        Err(_state) => return Err(error::Build::Load),
                    };

                    if new_ids {
                        let mut container = Container::new(&path);
                        container.inner.properties = container_state.properties.clone();
                        container.inner.assets = container_state
                            .assets
                            .clone()
                            .into_iter()
                            .map(|asset_state| {
                                let mut asset = Asset::new(asset_state.path.clone());
                                asset.properties = asset_state.properties;
                                asset
                            })
                            .collect();

                        container
                    } else {
                        container_state
                    }
                } else {
                    Container::new(&path)
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
                            path::Component::Normal(seg) => seg == app_dir(),
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
                    .iter()
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
                        container.assets.push(asset);
                    }
                }

                if let Err(err) = container.save() {
                    let err = match err {
                        error::Save::CreateDir(error) => error,
                        error::Save::SaveFiles {
                            properties,
                            assets,
                            settings,
                        } => {
                            if let Some(err) = properties {
                                err
                            } else if let Some(err) = assets {
                                err
                            } else {
                                settings.unwrap()
                            }
                        }
                    };

                    return Err(error::Build::Save(err.kind()));
                }

                if recurse {
                    for dir_path in dirs.into_iter().filter(|path| !ignore.contains(path)) {
                        init_container(dir_path, init_assets, new_ids, recurse, ignore)?;
                    }
                }

                Ok(container.rid().clone())
            }

            // main
            let path = path.as_ref();
            if !path.is_dir() {
                return Err(error::Build::NotADirectory);
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

            init_container(
                path,
                self.init_assets,
                self.init.new_ids,
                self.init.recursive,
                &ignore,
            )
        }
    }
}

pub mod error {
    use serde::{Deserialize, Serialize};
    use std::io;

    #[derive(Serialize, Deserialize, Debug, derive_more::From)]
    pub enum Build {
        Load,
        Save(#[serde(with = "io_error_serde::ErrorKind")] io::ErrorKind),
        NotADirectory,

        /// The path is already a Syre resource.
        AlreadyResource,
    }

    #[derive(Debug)]
    pub enum Save {
        CreateDir(io::Error),
        SaveFiles {
            properties: Option<io::Error>,
            assets: Option<io::Error>,
            settings: Option<io::Error>,
        },
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;
