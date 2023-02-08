//! Creates a temporary folder to perform actions in.
use crate::Result;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about the temporary folder.
#[derive(Debug)]
pub struct TempDir {
    _dir: tempfile::TempDir,
    pub children: HashMap<PathBuf, Self>,
    pub files: HashMap<PathBuf, tempfile::NamedTempFile>,
}

impl TempDir {
    pub fn new() -> Result<Self> {
        let td = tempfile::tempdir()?;
        Ok(TempDir {
            _dir: td,
            children: HashMap::new(),
            files: HashMap::new(),
        })
    }

    pub fn path(&self) -> &Path {
        self._dir.path()
    }

    /// Create a subdirectory.
    pub fn mkdir(&mut self) -> Result<PathBuf> {
        let td = tempfile::tempdir_in(self._dir.path())?;
        let tdir = TempDir {
            _dir: td,
            children: HashMap::new(),
            files: HashMap::new(),
        };
        let c_path = tdir.path().to_path_buf();
        self.children.insert(c_path.clone(), tdir);

        Ok(c_path)
    }

    /// Add a file to the directory.
    ///
    /// # See also
    /// + `mkfile_with_extension`
    pub fn mkfile(&mut self) -> Result<PathBuf> {
        let f = tempfile::NamedTempFile::new_in(self._dir.path())?;
        let path = f.path().to_path_buf();
        self.files.insert(path.clone(), f);

        Ok(path)
    }

    /// Add a file to the directory with a given extension.
    ///
    /// # See also
    /// + `mkfile`
    pub fn mkfile_with_extension<S: AsRef<OsStr>>(&mut self, ext: S) -> Result<PathBuf> {
        let f = tempfile::NamedTempFile::new_in(self._dir.path())?;
        let mut dst = f.path().to_path_buf();
        dst.set_extension(ext);

        fs::rename(f.path(), &dst)?;
        self.files.insert(dst.clone(), f);

        Ok(dst)
    }
}

impl Default for TempDir {
    fn default() -> Self {
        Self::new().expect("could not create tmp dir")
    }
}

#[cfg(test)]
#[path = "temp_dir_test.rs"]
mod temp_dir_test;
