//! Functionality to handle Scripts at a system level.
use super::collections::scripts::Scripts;
use crate::Result;
use std::path::Path;
use std::{fs, io};
use thot_core::project::Script;
use thot_core::types::{ResourceId, ResourcePath};
use thot_core::Error as CoreError;
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

    let abs_path = ResourcePath::new(fs::canonicalize(file)?)?;
    let script = match Script::new(abs_path) {
        Ok(script) => script,
        Err(err) => return Err(CoreError::ScriptError(err).into()),
    };
    let rid = script.rid.clone();

    let mut scripts = Scripts::load()?;
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

#[cfg(test)]
#[path = "./scripts_test.rs"]
mod scripts_test;
