use super::*;
use crate::settings::Settings;
use crate::types::Priority;
use crate::{Error, Result};
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use fake::faker::filesystem::raw::FileName;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

// *************
// *** setup ***
// *************

#[derive(Serialize, Deserialize, Derivative, Default)]
#[derivative(Debug)]
pub struct MockUserSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _rel_path: Option<PathBuf>,

    pub id: String,
}

impl MockUserSettings {
    pub fn new() -> Self {
        let id: Vec<String> = Words(EN, 10..11).fake();
        let id = id.join("-");

        MockUserSettings {
            _file_lock: None,
            _rel_path: None,
            id,
        }
    }
}

impl Settings for MockUserSettings {
    fn store_lock(&mut self, file_lock: FlockLock<File>) {
        self._file_lock = Some(file_lock);
    }

    fn file(&self) -> Option<&File> {
        match self._file_lock.as_ref() {
            None => None,
            Some(lock) => Some(&*lock),
        }
    }

    fn file_mut(&mut self) -> Option<&mut File> {
        match self._file_lock.as_mut() {
            None => None,
            Some(lock) => Some(&mut *lock),
        }
    }

    fn priority(&self) -> Priority {
        Priority::User
    }
}

impl UserSettings for MockUserSettings {
    fn base_path() -> Result<PathBuf> {
        Ok(std::env::temp_dir())
    }

    fn rel_path(&self) -> Result<PathBuf> {
        if self._rel_path.is_none() {
            return Err(Error::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "rel path not set",
            )));
        }

        let path = self._rel_path.clone().unwrap();
        Ok(path)
    }

    fn set_rel_path(&mut self, path: PathBuf) -> Result {
        self._rel_path = Some(path);
        Ok(())
    }
}

impl LockSettingsFile for MockUserSettings {}

// *************
// *** tests ***
// *************

// ---------------------
// --- User Settings ---
// ---------------------

#[test]
fn user_settings_path_should_concatenate_base_and_relative_paths() {
    let rel_path = PathBuf::from(FileName(EN).fake::<String>());
    let mut sets = MockUserSettings::new();
    sets.set_rel_path(rel_path.clone())
        .expect("set_rel_path should work");

    let base_path = MockUserSettings::base_path().expect("base path should be set");
    let sets_path = sets.path().expect("path should return a value");
    assert_eq!(base_path.join(rel_path), sets_path, "incorrect path");
}

#[test]
fn user_settings_load_should_work() {
    // setup
    let filename = PathBuf::from(FileName(EN).fake::<String>());
    let mut sets = MockUserSettings::new();
    let sets_id = sets.id.clone();
    sets.set_rel_path(filename.clone())
        .expect("set_rel_path should work");

    sets.acquire_lock().expect("acquire lock should work");
    sets.save().expect("save should work");
    drop(sets);

    // test
    let loaded = MockUserSettings::load(&filename).expect("load should work");
    assert_eq! {
        sets_id, loaded.id,
        "incorrect settings loaded"
    };
}

#[test]
fn user_settings_save_should_work() {
    // setup
    let filename = PathBuf::from(FileName(EN).fake::<String>());
    let mut sets = MockUserSettings::new();
    sets.set_rel_path(filename.clone())
        .expect("set_rel_path should work");

    sets.acquire_lock().expect("acquire lock should work");
    sets.save().expect("save should work");

    // test
    let sets_path = sets.path().expect("path should work");
    let loaded_str = fs::read_to_string(sets_path).expect("settings file should be readable");

    let loaded: MockUserSettings =
        serde_json::from_str(loaded_str.as_str()).expect("value should be valid json");

    assert_eq! {
        sets.id, loaded.id,
        "incorrect settings loaded"
    };
}

// --------------------------
// --- Lock Settings File ---
// --------------------------

#[test]
fn lock_file_settings_acquire_lock_should_obtain_file_lock() {
    let filename = PathBuf::from(FileName(EN).fake::<String>());
    let mut settings = MockUserSettings::new();
    settings
        .set_rel_path(filename)
        .expect("set_rel_path should work");

    settings.acquire_lock().expect("acquire lock panicked");

    assert!(settings.file().is_some(), "acquire lock failed")
}

#[test]
fn lock_file_settings_acquire_lock_should_exit_silently_if_lock_already_obtained() {
    let filename = PathBuf::from(FileName(EN).fake::<String>());
    let mut settings = MockUserSettings::new();
    settings
        .set_rel_path(filename)
        .expect("set_rel_path should work");

    settings
        .acquire_lock()
        .expect("file lock could not be acquired");

    if let Err(_) = settings.acquire_lock() {
        assert!(false, "acquiring file lock failed");
    }
}
