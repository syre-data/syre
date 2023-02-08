use cluFlock::FlockLock;
use macrotest;
use serde::{Deserialize, Serialize};
use settings_manager::prelude::{Settings, SettingsPriority, SettingsResult};
use settings_manager_change_path::settings_path;
use std::fs::File;
use std::path::{Path, PathBuf};

// *************
// *** setup ***
// *************

#[derive(Default, Serialize, Deserialize)]
struct BasicSettings {
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
    #[allow(unused_variables)]
    fn store_lock(&mut self, lock: FlockLock<File>) {}

    fn controls_file(&self) -> bool {
        false
    }

    fn priority(&self) -> SettingsPriority {
        SettingsPriority::System
    }
}

// *************
// *** tests ***
// *************

// --- integration tests ---

#[test]
#[settings_path("s")]
fn path_should_be_changed_randomly_if_not_provided() {
    let o_path = PathBuf::from("/tmp/my_temp_settings.json");

    let mut s = BasicSettings::default();
    s.set_path(o_path.clone());

    let r_path: SettingsResult<PathBuf> = s.path();
    let r_path = match r_path {
        Ok(p) => p,
        Err(err) => panic!("{:?}", err),
    };

    assert_ne!(o_path, r_path, "path was not changed");
}

#[test]
#[settings_path("s", "/tmp/new_path.json")]
fn path_should_be_changed_to_that_provided() {
    let n_path = Path::new("/tmp/new_path.json");
    let o_path = PathBuf::from("/tmp/my_temp_settings.json");

    let mut s = BasicSettings::default();
    s.set_path(o_path.clone());

    let r_path: SettingsResult<PathBuf> = s.path();
    let r_path = match r_path {
        Ok(p) => p,
        Err(err) => panic!("{:?}", err),
    };

    assert_ne!(o_path, r_path, "path was not changed");
    assert_eq!(n_path, r_path, "path was not changed to the one provided");
}

#[test]
#[settings_path("s")]
fn random_path_should_not_change_across_calls() {
    let mut s = BasicSettings::default();

    let r1_path: SettingsResult<PathBuf> = s.path();
    let r1_path = match r1_path {
        Ok(p) => p,
        Err(err) => panic!("{:?}", err),
    };

    let r2_path: SettingsResult<PathBuf> = s.path();
    let r2_path = match r2_path {
        Ok(p) => p,
        Err(err) => panic!("{:?}", err),
    };

    assert_eq!(r1_path, r2_path, "path changed across calls");
}

// #[test]
// #[settings_path("s")]
// fn should_expand_in_match_statement() {
//     let o_path = PathBuf::from("/tmp/my_temp_settings.json");
//
//     let mut s = BasicSettings::default();
//     s.set_path(o_path.clone());
//
//     let r_path: PathBuf = match s.path() {
//         Ok(p) => p,
//         Err(err) => panic!("{:?}", err),
//     };
//
//     assert_ne!(o_path, r_path, "path was not changed");
// }
//
// #[test]
// #[should_panic(expected = "Must provide variable name")]
// #[settings_path]
// fn should_panic_if_variable_name_not_given() {}

// --- expansion tests ---

#[test]
fn expansion_tests() {
    macrotest::expand("tests/expand/*.rs");
}
