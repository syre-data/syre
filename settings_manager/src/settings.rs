use crate::types::Priority;
use crate::{Error, Result};
use cluFlock::{ExclusiveFlock, FlockLock};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;
use std::default::Default;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;

/// Settings lock their respective file while in existence.
pub trait Settings<S>
where
    S: Serialize + DeserializeOwned + Clone,
{
    /// The settings.
    fn settings(&self) -> Cow<S>;

    /// The settings file if it is controlled.
    fn file(&self) -> &File;

    /// A mutable reference to the settings file if it is controlled.
    fn file_mut(&mut self) -> &mut File;

    /// The file lock.
    fn file_lock(&self) -> &FlockLock<File>;

    /// The priority of the settings object.
    fn priority(&self) -> Priority;

    /// Save the current object to a file.
    fn save(&mut self) -> Result {
        {
            // delete all data
            let file = self.file_mut();
            file.set_len(0)?;
            file.rewind()?;
        }

        serde_json::to_writer_pretty(self.file(), &*self.settings())
            .map_err(|err| Error::SerdeError(err))?;

        Ok(())
    }
}

/// Create a new settings object from a file.
/// Creates a default object of the type if the file did not exist or is empty.
pub fn load_or_create<T: Serialize + DeserializeOwned + Default>(
    path: &Path,
) -> Result<(T, FlockLock<File>)> {
    // get settings file and lock
    let settings_file = ensure_file(path)?;
    let file_lock = lock(settings_file)?;

    // get current settings
    let mut reader = BufReader::new(file_lock.as_ref());
    let mut settings_str = String::new();
    reader.read_to_string(&mut settings_str)?;

    let settings: T = if settings_str.is_empty() {
        // no content in file, create default
        let default = T::default();
        serde_json::to_writer_pretty(&*file_lock, &default)?;
        default
    } else {
        serde_json::from_str(&settings_str)?
    };

    Ok((settings, file_lock))
}

/// Obtain an exclusive file lock on the system settings file
/// to prevent other programs from accessing it.
pub fn lock(file: File) -> Result<FlockLock<File>> {
    match ExclusiveFlock::wait_lock(file) {
        Ok(lock) => Ok(lock),
        Err(flock_err) => Err(Error::IoError(flock_err.into_err())),
    }
}

/// Returns a file, ensuring it exists by creating it if needed.
pub fn ensure_file(path: &Path) -> Result<File> {
    // ensure settings directory exists
    let settings_dir = match path.parent() {
        Some(path) => path,
        None => {
            return Err(Error::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "invalid path",
            )))
        }
    };

    match fs::create_dir_all(settings_dir) {
        Ok(()) => {}                                                     // ok, continue
        Err(ref err) if err.kind() == io::ErrorKind::AlreadyExists => {} // directories already exist, continue
        Err(err) => return Err(Error::IoError(err)),
    }

    // create file if needed
    let file_res = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path);

    match file_res {
        Ok(file) => Ok(file),
        Err(err) => Err(Error::IoError(err)),
    }
}

#[cfg(test)]
#[path = "./settings_test.rs"]
mod settings_test;
