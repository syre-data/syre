use super::*;
use crate::settings::Settings;
use crate::types::Priority;
use crate::Result;
use cluFlock::FlockLock;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

// **********************
// ***  Mock Settings ***
// **********************

const SETTINGS_FILE: &str = "mock_system_settings.json";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MockSystemSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    id: String,
}

impl MockSystemSettings {
    pub fn new() -> Self {
        let id: Vec<String> = Words(EN, 10..11).fake();
        let id = id.join("-");

        MockSystemSettings {
            _file_lock: None,
            id,
        }
    }
}

impl Settings for MockSystemSettings {
    fn store_lock(&mut self, file_lock: FlockLock<File>) {
        self._file_lock = Some(file_lock);
    }

    fn controls_file(&self) -> bool {
        self._file_lock.is_some()
    }

    fn priority(&self) -> Priority {
        Priority::User
    }
}

impl SystemSettings for MockSystemSettings {
    fn path() -> Result<PathBuf> {
        let path = env::temp_dir().join(SETTINGS_FILE);
        Ok(path)
    }
}

impl LockSettingsFile for MockSystemSettings {}

// *************
// *** tests ***
// *************

// -----------------------
// --- System Settings ---
// -----------------------

#[test]
fn system_settings_path_should_work() {
    let file = MockSystemSettings::path().expect("path should work");
    assert!(file.ends_with(SETTINGS_FILE), "incorrect settings file");
}

#[test]
fn system_settings_load_should_work() {
    // setup
    let mut sets = MockSystemSettings::new();
    sets.acquire_lock().expect("acquire lock should work");
    sets.save().expect("save should work");

    // test
    let loaded = MockSystemSettings::load().expect("load should work");
    assert_eq! {
        sets.id, loaded.id,
        "incorrect settings loaded"
    };
}

#[test]
fn system_settings_save_should_work() {
    // setup
    let mut sets = MockSystemSettings::new();
    sets.acquire_lock().expect("acquire lock should work");
    sets.save().expect("save should work");

    // test
    let sets_path = MockSystemSettings::path().expect("path should work");
    let loaded_str = fs::read_to_string(sets_path).expect("settings file should be readable");
    let loaded: MockSystemSettings =
        serde_json::from_str(loaded_str.as_str()).expect("value should be valid json");

    assert_eq! {
        sets.id, loaded.id,
        "incorrect settings loaded"
    };
}

// --------------------------
// --- Lock File Settings ---
// --------------------------

#[test]
fn lock_file_settings_acquire_lock_should_obtain_file_lock() {
    let mut settings = MockSystemSettings::new();
    settings.acquire_lock().expect("acquire lock panicked");

    assert!(settings.controls_file(), "acquire lock failed")
}

#[test]
fn lock_file_settings_acquire_lock_should_exit_silently_if_lock_already_obtained() {
    let mut settings = MockSystemSettings::new();
    settings
        .acquire_lock()
        .expect("file lock could not be acquired");

    if let Err(_) = settings.acquire_lock() {
        assert!(false, "acquiring file lock failed");
    }
}
