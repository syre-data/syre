use super::*;
use crate::types::Priority;
use cluFlock::{ExclusiveFlock, FlockLock};
use fake::faker::name::raw::Name;
use fake::locales::EN;
use fake::Fake;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

// *************
// *** setup ***
// *************

///  Create settings file for BasicSettings
fn create_basic_settings_file() -> NamedTempFile {
    let mut settings = BasicSettings::default();
    settings.name = Name(EN).fake(); // randomize name

    let settings_file = NamedTempFile::new().expect("could not create temporary file");

    serde_json::to_writer_pretty(&settings_file, &settings)
        .expect("could not save settings to file");

    settings_file
}

#[derive(Default, Serialize, Deserialize)]
struct BasicSettings {
    #[serde(skip)]
    _file_lock: Option<FlockLock<File>>,

    #[serde(skip)]
    _path: PathBuf,

    pub name: String,
    pub age: u8,
}

impl BasicSettings {
    fn set_path(&mut self, path: PathBuf) {
        self._path = path;
    }

    fn path(&self) -> &Path {
        &self._path
    }
}

impl Settings for BasicSettings {
    fn store_lock(&mut self, lock: FlockLock<File>) {
        self._file_lock = Some(lock);
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
        Priority::System
    }
}

// *************
// *** tests ***
// *************

#[test]
fn load_should_work() {
    let settings_file = create_basic_settings_file();
    let _settings: BasicSettings = match load(settings_file.path()) {
        Ok(sets) => sets,
        Err(err) => {
            return assert!(false, "could not load file: {:?}", err);
        }
    };
}

#[test]
fn save_should_work() {
    // setup
    let mut settings = BasicSettings::default();
    let name: String = Name(EN).fake();
    let age = 10;
    settings.name = name.clone();
    settings.age = age.clone();

    let settings_file = NamedTempFile::new().expect("could not create settings file");
    let settings_path = settings_file.path().to_path_buf();
    settings.set_path(settings_path.clone());
    let file_lock =
        ExclusiveFlock::wait_lock(settings_file.reopen().expect("could not reopen file"))
            .expect("could not lock file");

    settings.store_lock(file_lock);

    // test
    save(&mut settings).expect("should not cause error");

    let mut f = File::open(&settings_path).unwrap();
    let mut res = String::new();
    f.read_to_string(&mut res).expect("could not read file");
    let res: BasicSettings = serde_json::from_str(&res).expect("could not convert string to JSON");

    assert_eq!(name, res.name, "names don't match");
    assert_eq!(age, res.age, "ages don't match");
}

#[test]
fn file_locking_should_work() {
    // setup
    let mut s1 = BasicSettings::default();
    let mut s2 = BasicSettings::default();
    let sf = NamedTempFile::new().expect("could not create settings file");

    let (sf, path) = sf.into_parts();
    s1.set_path(path.to_path_buf());
    s2.set_path(path.to_path_buf());

    // lock file
    let fl1 = ExclusiveFlock::wait_lock(sf).expect("could not lock file");

    s1.store_lock(fl1);
    assert!(s1.file().is_some(), "initial file lock was not obtained");

    // test
    // new thread for second file access
    let t = thread::spawn(move || {
        let sf2 = File::open(s2.path()).expect("could not open file");

        let fl2 = ExclusiveFlock::wait_lock(sf2).expect("could not lock file after initial");

        s2.store_lock(fl2);
        assert!(s2.file().is_some(), "second file lock was not obtained");
    });

    // sleep before drop
    thread::sleep(Duration::from_millis(500));
    drop(s1);

    t.join().expect("couldn't join");
}
