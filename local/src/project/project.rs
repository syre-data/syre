//! Functionality and resources related to projects.
use super::resources::{Analyses, Project};
use crate::common;
use crate::error::{Error, Project as ProjectError, Result};
use crate::system::collections::ProjectManifest;
use crate::system::project_manifest;
use std::path::{Path, PathBuf};
use std::{fs, io};
use syre_core::error::{Error as CoreError, Project as CoreProjectError};
use syre_core::project::Project as CoreProject;
use syre_core::types::ResourceId;

// ************
// *** Init ***
// ************

/// Initialize a new Syre project.
/// If the path is already initialized as a Syre resource -- i.e. has an app folder -- nothing is
/// done.
///
/// # Steps
/// 1. Create app folder to store data.
/// 2. Create [`Project`] for project info.
/// 3. Create `ProjectSettings` for project settings.
/// 4. Create `Script`s registry.
/// 5. Add [`Project`] to collections registry.
pub fn init(path: impl AsRef<Path>) -> Result<ResourceId> {
    let path = path.as_ref();
    if path_is_resource(path) {
        // project already initialized
        let rid = match project_id(path)? {
            Some(rid) => rid,
            None => {
                return Err(ProjectError::PathNotRegistered(path.to_path_buf()).into());
            }
        };

        return Ok(rid);
    }

    // create directory
    let syre_dir = common::app_dir_of(path);
    fs::create_dir(&syre_dir)?;

    // create app files
    let project = Project::new(path)?;
    project.save()?;

    let scripts = Analyses::new(path.into());
    scripts.save()?;

    project_manifest::register_project(project.base_path())?;
    Ok(project.rid.clone().into())
}

/// Creates a new Syre project.
///
/// # Errors
/// + If the folder already exists.
///
/// # See also
/// + [`init`]
pub fn new(root: &Path) -> Result<ResourceId> {
    if root.exists() {
        return Err(io::Error::new(io::ErrorKind::IsADirectory, "folder already exists").into());
    }

    fs::create_dir_all(root)?;
    init(root)
}

/// Move project to a new location.
pub fn mv(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Result {
    let from = from.into();
    let mut projects = ProjectManifest::load()?;
    if !projects.contains(&from) {
        return Err(ProjectError::PathNotAProjectRoot(from).into());
    }

    // move folder
    let to = to.into();
    if let Err(err) = fs::rename(&from, &to) {
        return Err(err.into());
    }

    projects.remove(&from);
    projects.push(to);
    projects.save()?;
    Ok(())
}

/// Returns whether the given path is part of a Syre project.
///
/// # Returns
/// `true`` if the path has a <APP_DIR> folder in it.
///
/// # Note
/// + Only works with `Container`s and `Project`s, not `Asset`s.
pub fn path_is_resource(path: &Path) -> bool {
    let path = common::app_dir_of(path);
    path.exists()
}

/// Returns whether the given path is a project root,
/// Deteremined by the presence of a project's properties file.
/// i.e. has a <APP_DIR>/<PROJECT_FILE>.
pub fn path_is_project_root(path: impl AsRef<Path>) -> bool {
    let path = common::project_file_of(path);
    path.exists()
}

/// Returns path to the project root.
///
/// # See also
/// + [`project_resource_root_path`]
pub fn project_root_path(path: impl AsRef<Path>) -> Option<PathBuf> {
    let mut path = path.as_ref().join("tmp"); // false join to pop off in loop
    while path.pop() {
        if path_is_project_root(&path) {
            return Some(path);
        }
    }

    None
}

/// Returns path to the project root for a Syre resource.
/// The entire path from start to the root of the project must follow resources.
/// i.e. If the path from start to root contains a folder that is not initiailized
/// as a Container, an error will be returned.
///
/// # See also
/// + [`project_root_path`]
pub fn project_resource_root_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    if !path_is_resource(path) {
        return Err(Error::Project(ProjectError::PathNotInProject(
            PathBuf::from(path),
        )));
    }

    let mut path = path.join("tmp"); // false join to pop off in loop
    while path.pop() {
        let prj_file = common::project_file_of(&path);
        if !prj_file.exists() {
            // folder is not root
            continue;
        }

        let Ok(prj_json) = fs::read_to_string(prj_file) else {
            // TODO Handle metalevel.
            // Currently assumed that if project file can't be read, it is because
            // the file is being controlled by another process, likely the database
            // so just return the path.
            return Ok(fs::canonicalize(path)?);
        };

        let prj: CoreProject = match serde_json::from_str(prj_json.as_str()) {
            Ok(prj) => prj,
            Err(err) => return Err(err.into()),
        };

        if prj.meta_level == 0 {
            return Ok(fs::canonicalize(path)?);
        }
    }

    Err(CoreError::Project(CoreProjectError::misconfigured("project has no root.")).into())
}

/// # Returns
/// + [`ResourceId`] of the containing [`Project`] if it exists.
/// + `None` if the path is not inside a `Project``.
pub fn project_id(path: impl AsRef<Path>) -> Result<Option<ResourceId>> {
    let root = match project_resource_root_path(path.as_ref()) {
        Ok(root) => root,
        Err(Error::Project(ProjectError::PathNotInProject(_))) => return Ok(None),
        Err(err) => return Err(err),
    };

    let project = Project::load_from(root)?;
    Ok(Some(project.rid.clone()))
}

pub mod converter {
    use super::super::container;
    use super::super::resources::{Analyses, Project};
    use crate::common;
    use crate::error::{Error, Project as ProjectError, Result};
    use crate::loader::container::Loader as ContainerLoader;
    use crate::system::project_manifest;
    use crate::system::settings;
    use std::collections::HashMap;
    use std::path::{Component, Path, PathBuf};
    use std::{fs, io};
    use syre_core::project::{AnalysisAssociation, Script, ScriptLang};
    use syre_core::types::{Creator, ResourceId, UserId, UserPermissions};

    pub struct Converter {
        data_root: PathBuf,
        analysis_root: Option<PathBuf>,
    }

    impl Converter {
        /// Creates a new converter.
        ///
        /// # Notes
        /// + `data_root` defaults to `data`.
        /// + `analysis_root` defaults to `analysis`.
        pub fn new() -> Self {
            Self {
                data_root: PathBuf::from("data"),
                analysis_root: Some(PathBuf::from("analysis")),
            }
        }

        pub fn set_data_root(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
            let path = path.into();
            Self::check_path(&path)?;
            if let Some(analysis_root) = self.analysis_root.as_ref() {
                if path.starts_with(analysis_root) || analysis_root.starts_with(&path) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidFilename,
                        "`data_root` and `analysis_root` must be distinct",
                    ));
                }
            }

            self.data_root = path;
            Ok(())
        }

        /// Indicates analysis scripts should be moved into the given folder and processed.
        pub fn with_scripts(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
            let path = path.into();
            Self::check_path(&path)?;
            if path.starts_with(&self.data_root) || self.data_root.starts_with(&path) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidFilename,
                    "`data_root` and `analysis_root` must be distinct",
                ));
            }

            self.analysis_root = Some(path);
            Ok(())
        }

        /// Do not initialize analysis scripts.
        pub fn without_scripts(&mut self) {
            self.analysis_root = None;
        }

        /// Converts an existing folder of data and scripts into a project.
        /// Registers the project in the project manifest.
        ///
        /// # Errors
        /// + If the path is already a `Project`.
        pub fn convert(&self, root: impl AsRef<Path>) -> Result<ResourceId> {
            let root = fs::canonicalize(root.as_ref())?;

            // create and register project
            let pid = match super::project_id(&root)? {
                Some(_id) => return Err(ProjectError::DuplicatePath(root).into()),
                None => match super::init(root.as_path()) {
                    Ok(rid) => {
                        let mut project = Project::load_from(root.as_path())?;
                        project.data_root = self.data_root.clone();
                        project.analysis_root = self.analysis_root.clone();

                        if let Ok(settings) = settings::UserSettings::load() {
                            let user = settings.active_user.clone().map(|user| UserId::Id(user));
                            project.creator = Creator::User(user);

                            if let Some(user) = settings.active_user.as_ref() {
                                project.settings_mut().permissions.insert(
                                    user.clone(),
                                    UserPermissions::with_permissions(true, true, true),
                                );
                            }
                        }

                        project.save()?;
                        rid
                    }

                    Err(Error::Project(ProjectError::PathNotRegistered(_path))) => {
                        let project = Project::load_from(&root)?;
                        project_manifest::register_project(root.clone())?;
                        project.rid.clone()
                    }

                    Err(err) => return Err(err),
                },
            };

            // create data and analysis roots
            // move contents into data root
            let tmp_dir = common::unique_file_name(root.join("__tmp__"))?;
            fs::create_dir(&tmp_dir)?;
            for entry in fs::read_dir(&root)? {
                let entry = entry?;
                let path = entry.path();
                if path == tmp_dir || path == common::app_dir_of(&root) {
                    continue;
                }

                let rel_path = path.strip_prefix(&root).unwrap();
                fs::rename(entry.path(), tmp_dir.join(rel_path))?;
            }

            let data_root = root.join(&self.data_root);
            fs::rename(tmp_dir, &data_root)?;

            if let Some(analysis_root) = self.analysis_root.as_ref() {
                // performed before intializing graph so scripts don't get registered as assets
                let analysis_root = root.join(analysis_root);
                fs::create_dir_all(&analysis_root)?;

                // move scripts
                #[cfg(target_os = "windows")]
                let data_root = common::strip_windows_unc(&data_root);

                let mut ext_pattern = data_root.join("**").join("*");
                let mut match_options = glob::MatchOptions::new();
                match_options.case_sensitive = false;

                let mut script_paths = Vec::new();
                for lang_ext in ScriptLang::supported_extensions() {
                    ext_pattern.set_extension(lang_ext);

                    for entry in
                        glob::glob_with(ext_pattern.to_str().unwrap(), match_options).unwrap()
                    {
                        let script_path = match entry {
                            Ok(path) => path,
                            Err(err) => return Err(err.into_error().into()),
                        };

                        let rel_path = script_path.strip_prefix(&data_root).unwrap().to_path_buf();
                        let to = analysis_root.join(&rel_path);
                        fs::create_dir_all(to.parent().unwrap())?;
                        fs::rename(script_path, to)?;
                        script_paths.push(rel_path);
                    }
                }

                // initialize scripts
                let mut scripts = Analyses::load_from(&root)?;
                for script_path in script_paths {
                    let Ok(script) = Script::from_path(script_path) else {
                        continue;
                    };

                    scripts
                        .insert_script_unique_path(script)
                        .map_err(|err| Error::Core(err.into()))?;
                }

                scripts.save()?;
            }

            // initialize container graph
            let mut builder = container::InitOptions::init();
            builder.recurse(true);
            builder.with_assets();
            builder.build(&data_root)?;

            if self.analysis_root.is_some() {
                // assign scripts
                let analyses = Analyses::load_from(&root)?;
                let mut container_scripts = HashMap::new();
                for script in analyses.scripts() {
                    let entry = container_scripts
                        .entry(script.path.parent().unwrap())
                        .or_insert(Vec::new());

                    entry.push(script.rid.clone());
                }

                for (container, scripts) in container_scripts {
                    let container = data_root.join(container);
                    let Ok(mut container) = ContainerLoader::load(container) else {
                        continue;
                    };

                    for script in scripts {
                        container.set_analysis_association(AnalysisAssociation::new(script));
                    }

                    container.save()?;
                }
            }

            Ok(pid)
        }

        fn check_path(path: impl AsRef<Path>) -> io::Result<()> {
            let path = path.as_ref();
            if !path.is_relative() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidFilename,
                    "path must be relative",
                ));
            }

            if path.components().any(|comp| comp == Component::ParentDir) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidFilename,
                    "path may not contain parent directory references (i.e. `..`)",
                ));
            }

            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;
