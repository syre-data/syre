use super::*;
use crate::types::ResourcePath;
use fake::faker::filesystem::raw::FilePath;
use fake::locales::EN;
use fake::Fake;
use rand::Rng;
use std::path::PathBuf;

// **************
// *** Script ***
// **************

#[test]
fn script_new_should_work() {
    // py
    let path = script_path(Some(String::from("py")));
    let script = Script::new(path.clone()).expect("creating script should work");
    assert_eq!(&path, &script.path, "script's path should be correct");

    // r
    let path = script_path(Some(String::from("R")));
    let script = Script::new(path.clone()).expect("creating script should work");
    assert_eq!(&path, &script.path, "script's path should be correct");
}

#[test]
#[should_panic(expected = "UnknownLanguage")]
fn script_new_with_unknown_file_type_should_error() {
    let path = script_path(Some(String::from("unknown")));
    Script::new(path).unwrap();
}

// ***************
// *** Scripts ***
// ***************

#[test]
fn scripts_new_should_work() {
    let _script = Scripts::new();
}

#[test]
fn scripts_contains_path_should_work() {
    // setup
    let scripts = create_scripts();
    let c_rid = scripts
        .keys()
        .next()
        .cloned()
        .expect("no `Script`s available");

    let script = scripts.get(&c_rid).expect("could not get `Script`");
    let c_path = script.path.clone();

    // test
    // known
    let found = scripts.contains_path(&c_path);
    assert!(found, "script should be found");

    // unknown
    let unknown = scripts.contains_path(&script_path(None));
    assert!(!unknown, "random script should not be found");
}

#[test]
fn scripts_by_path_should_work() {
    // setup
    let scripts = create_scripts();
    let c_rid = scripts
        .keys()
        .next()
        .cloned()
        .expect("no `Script`s available");

    let script = scripts.get(&c_rid).expect("could not get `Script`");
    let c_path = script.path.clone();

    // test
    // known
    let Some(found) = scripts.by_path(&c_path) else {
        panic!("script should be found");
    };

    assert_eq!(c_rid, found.rid, "correct script should be found");

    // unknown
    let unknown = scripts.by_path(&script_path(None));
    assert!(unknown.is_none(), "random script should not be found");
}

// ******************
// *** Script Env ***
// ******************

#[test]
fn script_env_new_should_work() {
    let script = script_path(Some("py".to_string()));
    let path = match script {
        ResourcePath::Absolute(path) => path,
        ResourcePath::Relative(path) => path,
        ResourcePath::Root(path, _) => path,
    };

    let _env = ScriptEnv::new(&path).expect("new should work");
}

#[test]
#[should_panic(expected = "UnknownLanguage")]
fn script_env_new_with_unknown_extension_should_error() {
    let script = script_path(Some("unknown".to_string()));
    let path = match script {
        ResourcePath::Absolute(path) => path,
        ResourcePath::Relative(path) => path,
        ResourcePath::Root(path, _) => path,
    };

    ScriptEnv::new(&path).unwrap();
}

// *******************
// *** Script Lang ***
// *******************

#[test]
fn script_lang_from_extension_should_work() {
    // py
    let py_lang = ScriptLang::from_extension(&OsStr::new("py"));
    assert_ne!(None, py_lang, "language should be found");
    let py_lang = py_lang.unwrap();
    assert_eq!(ScriptLang::Python, py_lang, "language should be correct");

    // r
    let r_lang = ScriptLang::from_extension(&OsStr::new("R"));
    assert_ne!(None, r_lang, "language should be found");
    let r_lang = r_lang.unwrap();
    assert_eq!(ScriptLang::R, r_lang, "language should be correct");
}

// ************************
// *** helper functions ***
// ************************

/// Selects a random path extension from a set of valid ones.
fn random_path_ext() -> String {
    let valid_ext = ["py", "R"];

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..valid_ext.len());

    let ext = valid_ext[index].to_string();
    ext
}

/// Creates a random path.
/// If `ext` is `None` uses a valid random extension.
/// If `ext` is `Some`, uses the given value.
fn script_path(ext: Option<String>) -> ResourcePath {
    let mut path;
    loop {
        // enusre path is a valid path, not root
        path = PathBuf::from(FilePath(EN).fake::<String>());
        if path.parent().is_some() {
            break;
        }
    }
    match ext {
        None => path.set_extension(random_path_ext()),
        Some(ext) => path.set_extension(ext),
    };

    ResourcePath::new(path).expect("creating resource path should work")
}

/// Creates a [`Script`] with a random path.
/// If `ext` is `None` uses a valid random extension.
/// If `ext` is `Some`, uses the given value.
fn create_script(ext: Option<String>) -> Result<Script> {
    let path = script_path(ext);
    Script::new(path)
}

/// Creates a [`Scripts`] with random script paths.
fn create_scripts() -> Scripts {
    let mut rng = rand::thread_rng();
    let n_scripts = rng.gen_range(1..20);
    let mut scripts = Scripts::new();
    for _ in 0..n_scripts {
        let script = create_script(None).expect("creating new script should work");
        scripts.insert(script.rid.clone(), script);
    }

    scripts
}
