//! Functionality to handle Scripts at a system level.
use super::collections::scripts::Scripts;
use crate::Result;
use settings_manager::SystemSettings;
use std::path::Path;
use std::{fs, io};
use thot_core::project::Script;
use thot_core::types::{ResourceId, ResourcePath};
use uuid::Uuid;

// **************
// *** Script ***
// **************

/// Make the given file a [`Script`].
pub fn make_script(file: &Path) -> Result<ResourceId> {
    if !file.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "script file does not exist").into());
    }

    if !file.is_file() {
        return Err(
            io::Error::new(io::ErrorKind::IsADirectory, "script file is not a file").into(),
        );
    }

    let abs_path = match fs::canonicalize(file) {
        Ok(path) => path,
        Err(err) => return Err(err.into()),
    };

    let abs_path = ResourcePath::new(abs_path)?;
    let script = Script::new(abs_path)?;
    let rid = script.rid.clone();

    let mut scripts = Scripts::load_or_default()?;
    scripts.insert(rid.clone(), script);
    scripts.save()?;

    // success
    Ok(rid)
}

pub fn r#move(id: Uuid, path: &Path) -> Result {
    todo!();
}

/// Finds a script given its path.
pub fn script_by_path(path: &Path) -> Result<Script> {
    todo!();
}

// @remove
///// Initialize a file as a Script.
/////
///// If `project` is `None`, assumes the file is contained within the project.
//pub fn init(path: &Path, project: Option<&Path>) -> Result<ResourceId> {
//    let project = match project {
//        None => project_root_path(path)?,
//        Some(p) => {
//            // check path is a project root
//            if !path_is_project_root(p) {
//                return Err(Error::ProjectError(ProjectError::PathNotAProjectRoot(
//                    p.to_path_buf(),
//                )));
//            }

//            p.to_path_buf()
//        }
//    };

//    let s_rp = ResourcePath::new(path.to_path_buf())?;
//    let mut scripts = Scripts::load()?;

//    // check if script is already registered
//    for s in &scripts.scripts {
//        if s.path == s_rp {
//            return Ok(s.rid.clone().into());
//        }
//    }

//    let script = Script::new(s_rp)?;
//    let rid = script.rid.clone();
//    scripts.push(script);
//    scripts.save()?;

//    Ok(rid.into())
//}

// @todo
// /// Returns the [`Script`]s associated with the path.
// /// If project path is not given, searches in ancestors for project root.
// /// Errors if the path is not registered.
// pub fn id_by_path(path: &Path, project: Option<&Path>) -> Result<ResourceId> {
//     // get project
//     let project = match project {
//         Some(p) => {
//             if !path_is_project_root(p) {
//                 return Err(Error::ProjectError(ProjectError::PathNotAProjectRoot(
//                     p.to_path_buf(),
//                 )));
//             }
//             p.to_path_buf()
//         }
//         None => {
//             let p = project_root_path(path)?;
//             p
//         }
//     };

//     let scripts = Scripts::load(&project)?;
//     let s_path = ResourcePath::new(path.to_path_buf())?;
//     match scripts.by_path(&s_path) {
//         None => Err(Error::CoreError(CoreError::ProjectError(
//             CoreProjectError::NotRegistered(None, Some(path.to_path_buf())),
//         ))),
//         Some(script) => Ok(script.rid.clone().into()),
//     }
// }

// @todo
// /// Returns `true` if the path is registered as a script with the given project.
// /// If project path is not given, searches in ancestors for project root.
// pub fn path_is_registered(path: &Path, project: Option<&Path>) -> Result<bool> {
//     // get project
//     let project = match project {
//         Some(p) => {
//             if !path_is_project_root(p) {
//                 return Err(Error::ProjectError(ProjectError::PathNotAProjectRoot(
//                     p.to_path_buf(),
//                 )));
//             }
//             p.to_path_buf()
//         }
//         None => {
//             let p = project_root_path(path)?;
//             p
//         }
//     };

//     // check if script is registered
//     let scripts = Scripts::load(&project)?;
//     let s_path = ResourcePath::new(path.to_path_buf())?;
//     Ok(scripts.contains_path(&s_path))
// }

#[cfg(test)]
#[path = "./scripts_test.rs"]
mod scripts_test;
