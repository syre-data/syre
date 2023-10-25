//! Creates a temporary folder to perform actions in.
use crate::Result;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about the temporary folder.
#[derive(Debug)]
pub struct TempDir {
    dir: tempfile::TempDir,
    pub children: HashMap<PathBuf, Self>,
    pub files: HashMap<PathBuf, tempfile::NamedTempFile>,
}

impl TempDir {
    pub fn new() -> Result<Self> {
        let td = tempfile::tempdir()?;
        Ok(TempDir {
            dir: td,
            children: HashMap::new(),
            files: HashMap::new(),
        })
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Create a subdirectory.
    pub fn mkdir(&mut self) -> Result<PathBuf> {
        let td = tempfile::tempdir_in(self.dir.path())?;
        let tdir = TempDir {
            dir: td,
            children: HashMap::new(),
            files: HashMap::new(),
        };
        let c_path = tdir.path().to_path_buf();
        self.children.insert(c_path.clone(), tdir);

        Ok(c_path)
    }

    /// Add a file to the directory.
    pub fn mkfile(&mut self) -> Result<PathBuf> {
        let f = tempfile::NamedTempFile::new_in(self.dir.path())?;
        let path = f.path().to_path_buf();
        self.files.insert(path.clone(), f);

        Ok(path)
    }

    /// Add a file to the directory with a given name.
    pub fn mkfile_with_name<S: AsRef<OsStr>>(&mut self, file_name: S) -> Result<PathBuf> {
        let f = tempfile::NamedTempFile::new_in(self.dir.path())?;
        let mut dst = f.path().to_path_buf();
        dst.set_file_name(file_name);

        fs::rename(f.path(), &dst)?;
        self.files.insert(dst.clone(), f);

        Ok(dst)
    }

    /// Add a file to the directory with a given extension.
    pub fn mkfile_with_extension<S: AsRef<OsStr>>(&mut self, ext: S) -> Result<PathBuf> {
        let f = tempfile::NamedTempFile::new_in(self.dir.path())?;
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
