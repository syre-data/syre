//! Functionality and resources related to Thot Projects.
//!
//! This includes:
//! + Projects
//! + Containers
//! + Assets
//! + Script Associations
pub mod asset;
pub mod container;
pub mod project;
pub mod resources;
pub mod script;

/// Current project format version.
pub static PROJECT_FORMAT_VERSION: &str = "0.10.0";

// *****************
// *** functions ***
// *****************

use crate::common;
use crate::error::{Error, Project as ProjectError, Result};
use crate::system::project_manifest;
use resources::Project;
use std::path::{Component, Path};
use std::{fs, io};
use thot_core::project::{Script, ScriptLang};
use thot_core::types::{ResourceId, ResourcePath};

/// Initializes an existing folder of data and scripts into a project.
pub fn init(
    root: impl AsRef<Path>,
    data_root: impl AsRef<Path>,
    analysis_root: impl AsRef<Path>,
) -> Result<ResourceId> {
    let root = root.as_ref();
    let data_root = data_root.as_ref();
    let analysis_root = analysis_root.as_ref();

    let root = fs::canonicalize(&root)?;
    if !data_root.is_relative() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidFilename,
            "data root path must be relative",
        )
        .into());
    }

    if !analysis_root.is_relative() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidFilename,
            "analysis root path must be relative",
        )
        .into());
    }

    if data_root
        .components()
        .any(|comp| comp == Component::ParentDir)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidFilename,
            "data root path may not containe parent directory references (e.g. `..`)",
        )
        .into());
    }

    if analysis_root
        .components()
        .any(|comp| comp == Component::ParentDir)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidFilename,
            "analysis root path may not contain parent directory references (e.g. `..`)",
        )
        .into());
    }

    // create and register project
    let pid = match project::project_id(root.as_path())? {
        Some(id) => id,
        None => match project::init(root.as_path()) {
            Ok(rid) => {
                let mut project = Project::load_from(root.as_path())?;
                project.data_root = Some(data_root.to_path_buf());
                project.analysis_root = Some(analysis_root.to_path_buf());
                project.save()?;
                rid
            }
            Err(Error::Project(ProjectError::PathNotRegistered(_path))) => {
                let project = Project::load_from(&root)?;
                project_manifest::register_project(project.rid.clone(), root.clone())?;
                project.rid.clone()
            }
            Err(err) => return Err(err),
        },
    };

    // initialize analysis scripts
    let data_root = root.join(data_root);
    let analysis_root = root.join(analysis_root);

    let mv_analysis = analysis_root.exists();
    fs::create_dir_all(&analysis_root)?;
    let analysis_root = fs::canonicalize(analysis_root)?;
    if !analysis_root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidFilename,
            "analysis root must be a directory",
        )
        .into());
    }

    if mv_analysis {
        for lang_ext in ScriptLang::supported_extensions() {
            let mut ext_pattern = root.join("**").join("*");
            ext_pattern.set_extension(lang_ext);

            let mut match_options = glob::MatchOptions::new();
            match_options.case_sensitive = false;

            for entry in glob::glob_with(ext_pattern.to_str().unwrap(), match_options).unwrap() {
                let script_path = match entry {
                    Ok(path) => path,
                    Err(err) => return Err(err.into_error().into()),
                };

                let to = script_path.strip_prefix(&root).unwrap();
                let to = analysis_root.join(to);
                fs::rename(script_path, to)?;
            }
        }
    }

    let mut scripts = resources::Scripts::load_from(&root)?;
    for entry in fs::read_dir(&analysis_root)? {
        let entry = entry?;
        let script_path = fs::canonicalize(entry.path())?;
        let script_path = script_path.strip_prefix(&analysis_root).unwrap();
        let script_path = ResourcePath::new(script_path.to_path_buf())?;
        if scripts.contains_path(&script_path) {
            continue;
        }

        let Ok(script) = Script::new(script_path) else {
            continue;
        };

        scripts.insert_script(script)?;
    }
    scripts.save()?;

    // initialize container graph
    let mv_data = !data_root.exists();
    fs::create_dir_all(&data_root)?;
    let data_root = fs::canonicalize(data_root)?;
    if mv_data {
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let entry_path = fs::canonicalize(entry.path())?;
            if entry_path == analysis_root
                || entry_path == data_root
                || entry_path == common::thot_dir_of(&root)
            {
                continue;
            }

            let data_path = entry_path.strip_prefix(&root).unwrap();
            let data_path = data_root.join(data_path);
            fs::rename(entry_path, data_path)?;
        }
    }

    let mut builder = container::InitOptions::init();
    builder.ignore(
        analysis_root
            .strip_prefix(&root)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    );
    builder.build(&data_root)?;

    Ok(pid)
}
