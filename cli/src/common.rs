use crate::Result;
use std::path::PathBuf;
use std::{env, fs};

/// Returns the absolute version of the path, relative to the current directory, if path is relative.
pub fn abs_path(mut path: PathBuf) -> Result<PathBuf> {
    if path.is_relative() {
        let cwd = env::current_dir()?;
        path = cwd.join(path);
    };

    path = fs::canonicalize(path)?;
    Ok(path)
}
