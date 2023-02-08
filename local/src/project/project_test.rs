use super::*;
use crate::common::{project_file_of, project_settings_file_of, thot_dir_of};
use crate::project::resources::project::Project;
use dev_utils::fs::TempDir;
use fake::faker::filesystem::raw::DirPath;
use fake::locales::EN;
use fake::Fake;
use serde_json;
use std::fs;

// ******************
// *** new + init ***
// ******************

#[test]
fn new_should_work() {
    let _dir = TempDir::new().expect("setup should work");
    let root = _dir.path().join("root");
    let root = root.as_path();

    // initial check that root dir does not exist
    assert_eq!(false, root.is_dir(), "root dir should not exist before new");

    // initialize thot project
    new(root).expect("new should work");

    let thot_dir = thot_dir_of(root);
    assert_eq!(true, root.is_dir(), "root dir should exist after new");
    assert_eq!(true, thot_dir.is_dir(), "thot dir should exist after new");

    // ensure project files created
    let prj_files = vec![project_file_of(root), project_settings_file_of(root)];
    for f in prj_files {
        assert!(f.is_file(), "{:?} file should exist after new", f);
    }

    // ensure meta level is 0
    let prj_file = project_file_of(root);
    let prj_json = fs::read_to_string(prj_file.as_path()).expect("could not read project file");
    let prj: Project = serde_json::from_str(&prj_json).expect("project should be valid json");
    assert_eq!(0, prj.meta_level, "project should have meta level 0");
}

#[test]
#[should_panic(expected = "IsADirectory")]
fn new_should_error_if_directory_already_exists() {
    let _dir = TempDir::new().expect("setup should work");
    new(_dir.path()).unwrap();
}

#[test]
fn init_should_work() {
    let _dir = TempDir::new().expect("setup should work");
    let root = _dir.path();

    // initialize thot project
    let init_res = init(root);
    assert_eq!(true, init_res.is_ok(), "new should return Ok");

    let thot_dir = thot_dir_of(root);
    assert_eq!(
        true,
        thot_dir.is_dir(),
        "thot directory should exist after init"
    );

    // ensure files created
    let prj_files = [project_file_of(root), project_settings_file_of(root)];
    for f in prj_files {
        assert!(f.is_file(), "{:?} file should exist after new", f);
    }

    // ensure project has meta level 0
    let prj_file = project_file_of(root);
    let prj_json = fs::read_to_string(prj_file).expect("could not read project file");
    let prj: Project = serde_json::from_str(&prj_json).expect("project should be valid json");
    assert_eq!(0, prj.meta_level, "project has incorrect meta level");
}

#[test]
fn init_if_thot_directory_exists_should_do_nothing() {
    // setup
    let _dir = TempDir::new().expect("setup should work");
    let root = _dir.path();
    let rid = init(root).expect("init should work");

    // test
    let found_rid = init(root).expect("init should work even if already initialized");
    assert_eq!(rid, found_rid, "resource ids should match");
}

#[test]
#[should_panic(expected = "NotFound")]
fn init_should_error_if_not_given_an_existing_directory() {
    let _dir = TempDir::new().expect("setup should work");
    let false_root = _dir.path().join("absent");

    init(false_root.as_path()).unwrap();
}

// ************
// *** move ***
// ************

#[test]
fn mv_moves_project_to_new_location_and_updates_resources() {
    // setup
    let _dir = TempDir::new().expect("setup should work");
    let o_root = _dir.path().join("orig");
    let rid = new(o_root.as_path()).expect("new should work");

    // move
    let n_root = _dir.path().join("new");
    assert!(!n_root.exists(), "new path already exists");

    mv(&rid, &n_root).expect("move should work");

    // test
    let prj = projects::project_by_id(&rid)
        .expect("project_by_id should work")
        .unwrap();
    assert_eq!(&n_root, &prj.path, "path was not updated in registry");
    assert!(n_root.exists(), "project was not moved to new path");

    let prj_file = project_file_of(&n_root);
    let prj_json = fs::read_to_string(prj_file).expect("could not read project file");
    let prj: Project =
        serde_json::from_str(&prj_json).expect("project file should be parsable json");

    assert_eq!(rid, prj.rid, "resource ids do not match");
}

// *************************
// *** path is resource ***
// *************************

#[test]
fn path_is_resource_should_work() {
    let _dir = TempDir::new().expect("setup should work");
    let root = _dir.path();

    assert_eq!(
        false,
        path_is_resource(root),
        "path should not be a resource"
    );

    init(root).expect("init should work");
    assert_eq!(true, path_is_resource(root), "path should be a resource");
}

// *************************
// *** project root path ***
// *************************

#[test]
fn project_root_path_should_work_for_root() {
    // setup
    let _dir = TempDir::new().expect("setup should work");
    let root = _dir.path();
    let rid = init(root).expect("init should work");

    // test
    let found = project_root_path(root).expect("project_root_path should work");

    let prj = projects::project_by_id(&rid)
        .expect("project_by_id should work")
        .unwrap();

    assert_eq!(prj.path, found, "project path is incorrect");
}

#[test]
fn project_root_path_should_work_for_descendents() {
    // setup
    let mut _dir = TempDir::new().expect("setup should work");
    init(_dir.path()).expect("init should work");
    let cp1 = _dir.mkdir().expect("making child directory should work");
    let c1 = _dir
        .children
        .get_mut(&cp1)
        .expect("child directory should be found");
    let cp2 = c1.mkdir().expect("making grandchild directory should work");

    // test
    let f1 = project_root_path(&cp1).expect("project root path should work");
    assert_eq!(
        _dir.path(),
        f1.as_path(),
        "found project root from child should be correct"
    );

    let f2 = project_root_path(&cp2).expect("project root path should work");
    assert_eq!(
        _dir.path(),
        f2.as_path(),
        "found project root from grandchild should be correct"
    );
}

#[test]
fn project_root_path_if_exits_resource_path_should_work() {
    todo!();
}

#[test]
#[should_panic(expected = "PathNotInProject")]
fn project_root_path_if_path_is_not_in_a_project_should_error() {
    let _dir = TempDir::new().expect("setup should work");
    project_root_path(_dir.path()).unwrap();
}

#[test]
#[should_panic(expected = "PathNotInProject")]
fn project_root_path_if_no_root_is_found_should_error() {
    todo!();
}

#[test]
fn project_resource_root_path_for_root_should_work() {
    // setup
    let _dir = TempDir::new().expect("setup should work");
    let root = _dir.path();
    let rid = init(root).expect("init should work");

    // test
    let found = project_resource_root_path(root).expect("project_root_path should work");

    let prj = projects::project_by_id(&rid)
        .expect("project_by_id should work")
        .unwrap();

    assert_eq!(prj.path, found, "project path is incorrect");
}

#[test]
fn project_resource_root_path_should_work_for_descendents() {
    todo!();
}

#[test]
#[should_panic(expected = "PathNotInProject")]
fn project_resource_root_path_should_error_if_path_is_not_in_a_project() {
    let root: PathBuf = DirPath(EN).fake();
    project_resource_root_path(root.as_path()).unwrap();
}

#[test]
#[should_panic(expected = "Misconfigured")]
fn project_resource_root_path_should_error_if_no_root_is_found() {
    todo!();
}

// ****************************
// *** project registration ***
// ****************************

#[test]
fn project_registration_should_work_for_root() {
    // setup
    let _dir = TempDir::new().expect("setup should work");
    let root = _dir.path();
    let rid = init(root).expect("init should work");

    // test
    let prj = project_registration(root).expect("project_registration should work");
    assert_eq!(rid, prj.rid, "ids do not match");
}

#[test]
fn project_registration_should_work_for_descendents() {
    todo!();
}

#[test]
#[should_panic(expected = "PathNotInProject")]
fn project_registration_should_error_if_path_is_not_in_project() {
    let root: PathBuf = DirPath(EN).fake();
    project_registration(root.as_path()).unwrap();
}

#[test]
#[should_panic(expected = "Misconfigured")]
fn project_registration_should_error_if_project_does_not_have_root() {
    todo!();
}
