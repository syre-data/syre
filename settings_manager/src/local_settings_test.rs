use super::*;
use crate::settings::Settings;
use crate::types::Priority;
use crate::{Error, Result};
use cluFlock::FlockLock;
use derivative::{self, Derivative};
use dev_utils::fs::TempDir;
use fake::faker::filesystem::raw::DirPath;
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
const FILENAME: &str = "mock_settings.json";

#[derive(Serialize, Deserialize, Derivative, Default)]
#[derivative(Debug)]
pub struct MockLocalSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _base_path: Option<PathBuf>,

    pub id: String,
}

impl MockLocalSettings {
    pub fn new() -> Self {
        let id: Vec<String> = Words(EN, 10..11).fake();
        let id = id.join("-");

        MockLocalSettings {
            _file_lock: None,
            _base_path: None,
            id,
        }
    }
}

impl Settings for MockLocalSettings {
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
        Priority::Local
    }
}

impl LocalSettings for MockLocalSettings {
    fn rel_path() -> Result<PathBuf> {
        Ok(PathBuf::from(FILENAME))
    }

    fn base_path(&self) -> Result<PathBuf> {
        if self._base_path.is_none() {
            return Err(Error::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "base path not set",
            )));
        }

        let path = self._base_path.clone().unwrap();
        Ok(path)
    }

    fn set_base_path(&mut self, path: PathBuf) -> Result {
        self._base_path = Some(path);
        Ok(())
    }
}

impl LockSettingsFile for MockLocalSettings {}

// *************
// *** tests ***
// *************

// -----------------------
// --- Local Settings ---
// -----------------------

#[test]
fn local_settings_path_should_concatenate_base_and_relative_paths() {
    let base_path: PathBuf = DirPath(EN).fake();
    let mut sets = MockLocalSettings::new();
    sets.set_base_path(base_path.clone())
        .expect("set_base_path should work");

    let sets_path = sets.path().expect("path should return a value");
    assert_eq!(base_path.join(FILENAME), sets_path, "incorrect path");
}

#[test]
fn local_settings_load_should_work() {
    // setup
    let _dir = TempDir::new().expect("setup should work");
    let mut sets = MockLocalSettings::new();
    let sets_id = sets.id.clone();
    sets.set_base_path(_dir.path().to_path_buf())
        .expect("set_base_path should work");

    sets.acquire_lock().expect("acquire lock should work");
    sets.save().expect("save should work");
    drop(sets);

    // test
    let loaded = MockLocalSettings::load(_dir.path()).expect("load should work");
    assert_eq! {
        sets_id, loaded.id,
        "incorrect settings loaded"
    };
}

#[test]
fn local_settings_save_should_work() {
    // setup
    let _dir = TempDir::new().expect("setup should work");
    let mut sets = MockLocalSettings::new();
    sets.set_base_path(_dir.path().to_path_buf())
        .expect("set_base_path should work");

    sets.acquire_lock().expect("acquire lock should work");
    sets.save().expect("save should work");

    // test
    let sets_path = sets.path().expect("path should work");
    let loaded_str = fs::read_to_string(sets_path).expect("settings file should be readable");

    let loaded: MockLocalSettings =
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
    let _dir = TempDir::new().expect("setup should work");
    let mut settings = MockLocalSettings::new();
    settings
        .set_base_path(_dir.path().to_path_buf())
        .expect("set_base_path should work");

    settings.acquire_lock().expect("acquire lock panicked");

    assert!(settings.file().is_some(), "acquire lock failed")
}

#[test]
fn lock_file_settings_acquire_lock_should_exit_silently_if_lock_already_obtained() {
    let _dir = TempDir::new().expect("setup should work");
    let mut settings = MockLocalSettings::new();
    settings
        .set_base_path(_dir.path().to_path_buf())
        .expect("set_base_path should work");

    settings
        .acquire_lock()
        .expect("file lock could not be acquired");

    if let Err(_) = settings.acquire_lock() {
        assert!(false, "acquiring file lock failed");
    }
}
