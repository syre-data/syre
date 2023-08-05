//! For temporary files.
use crate::Result;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

/// Creates a temporary file
///
/// # See also
/// + `mkfile_with_extension`
pub fn mkfile() -> Result<PathBuf> {
    let f = tempfile::NamedTempFile::new()?;
    let path = f.path().to_path_buf();

    Ok(path)
}

/// Creates a temporary file with the given extension.
///
/// # See also
/// + `mkfile`
pub fn mkfile_with_extension<S: AsRef<OsStr>>(ext: S) -> Result<PathBuf> {
    let f = tempfile::NamedTempFile::new()?;
    let mut dst = f.path().to_path_buf();
    dst.set_extension(ext);

    fs::rename(f.path(), &dst)?;
    Ok(dst)
}
