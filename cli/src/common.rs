use crate::result::Result;
use std::env;
use std::path::PathBuf;
use thot_local::common::canonicalize_path;

/// Returns the absolute version of the path, relative to the current directory, if path is relative.
pub fn abs_path(path: PathBuf) -> Result<PathBuf> {
    let mut path = path;
    if path.is_relative() {
        let cwd = env::current_dir()?;
        path = cwd.join(path);
    };

    path = canonicalize_path(path)?;
    Ok(path)
}
