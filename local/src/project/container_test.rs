use super::*;
use crate::common::thot_dir_of;
use crate::constants::{ASSETS_FILE, CONTAINER_FILE, CONTAINER_SETTINGS_FILE, THOT_DIR};
use crate::project::resources::Container;
use dev_utils::fs::TempDir;
use fake::faker::filesystem::raw::FileName;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;
use serde_json;
use std::fs;
use thot_core::project::container::{AssetMap, Container as CoreContainer};

#[test]
fn init_on_non_resource_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let root = _dir.path().join("container");
    fs::create_dir(&root).expect("create directory should work");

    // test
    init(root.as_path()).expect("init should work");

    // check .thot folder created
    let mut thot_dir = root.clone();
    thot_dir.push(THOT_DIR);
    assert!(thot_dir.exists(), ".thot folder should exist");

    // check files exist and are empty
    let files = vec![CONTAINER_FILE, CONTAINER_SETTINGS_FILE, ASSETS_FILE];
    for f in files {
        let p = thot_dir.join(f);
        assert!(p.exists(), "{} file should exist", f);
    }

    // ensure assets are empty
    let assets_path = thot_dir.join(ASSETS_FILE);
    let assets_json = fs::read_to_string(assets_path).expect("assets file should be readable");
    let assets: AssetMap =
        serde_json::from_str(assets_json.as_str()).expect("assets should be valid json");

    assert_eq!(0, assets.len(), "assets not empty");

    // ensure container info is correct
    let container_path = thot_dir.join(CONTAINER_FILE);
    let container_json =
        fs::read_to_string(container_path).expect("container file should be readable");
    let container: CoreContainer =
        serde_json::from_str(container_json.as_str()).expect("container should be valid json");

    assert_eq!(0, container.assets.len(), "assets should be empty");
    assert_eq!(0, container.scripts.len(), "scripts should be empty");
}

#[test]
fn init_should_return_resource_id_if_already_a_container() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let rid = init(_dir.path()).expect("init should work");

    // test
    let found_rid = init(_dir.path()).expect("init should return old resource id");
    assert_eq!(rid, found_rid, "resource ids should match");
}

#[test]
fn init_should_work_if_folder_is_a_thot_resource_but_not_a_container() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    fs::create_dir(thot_dir_of(_dir.path())).expect("creating thot directory should work");

    // test
    init(_dir.path()).expect("init should work");
}

#[test]
#[should_panic(expected = "NotADirectory")]
fn init_should_error_if_folder_does_not_exist() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let path = _dir.path().join("root");

    // test
    init(path.as_path()).unwrap();
}

#[test]
fn init_from_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    let container = CoreContainer::new(name);

    // test
    init_from(_dir.path(), container.clone()).expect("`init_from` should work");
    let c = Container::load_from(_dir.path()).expect("could not load `Container`");
    assert_eq!(container.rid, c.rid, "`rid` does not match");
    assert_eq!(
        container.properties, c.properties,
        "`properties` do not match"
    );
    // @todo: Ensure `children`, `assets`, and `scripts` match.
}

#[test]
fn new_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");

    // test
    let c_path = _dir.path().join("container");
    new(c_path.as_path()).expect("new should work");
}

#[test]
#[should_panic(expected = "IsADirectory")]
fn new_should_error_if_path_already_exists() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");

    // test
    new(_dir.path()).unwrap();
}

#[test]
fn init_child_with_default_container_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let child = child_dir(_dir.path());
    fs::create_dir(&child).expect("creating child folder should work");
    init(_dir.path()).expect("init parent should work");

    // test
    init_child(&child, None).expect("init as child should work");
}

#[test]
fn init_child_with_specified_container_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let child = child_dir(_dir.path());
    fs::create_dir(&child).expect("creating child folder should work");
    init(_dir.path()).expect("init parent should work");

    // test
    init_child(&child, Some(_dir.path())).expect("init as child should work");
}

#[test]
#[should_panic(expected = "NotAContainer")]
fn init_child_should_error_if_parent_is_not_a_container() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let child = child_dir(_dir.path());
    fs::create_dir(&child).expect("creating child folder should work");

    // test
    init_child(&child, None).unwrap();
}

#[test]
#[should_panic(expected = "NotADirectory")]
fn init_child_should_error_if_path_does_not_exist() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let child = child_dir(_dir.path());
    init(_dir.path()).expect("init parent should work");

    // test
    init_child(&child, None).unwrap();
}

#[test]
#[should_panic(expected = "InvalidChildPath")]
fn init_child_should_error_if_child_is_not_a_child_folder_of_parent() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let child = child_dir(&child_dir(_dir.path()));

    fs::create_dir_all(&child).expect("create child folder should work");
    init(_dir.path()).expect("init parent should work");

    // test
    init_child(&child, Some(_dir.path())).unwrap();
}

#[test]
fn new_child_with_default_container_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    init(_dir.path()).expect("init parent should work");
    let child = child_dir(_dir.path());

    // test
    let _rid = new_child(&child, None).expect("new child should work");
}

#[test]
fn new_child_with_specified_container_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    init(_dir.path()).expect("init parent should work");
    let child = child_dir(_dir.path());

    // test
    let _rid = new_child(&child, Some(_dir.path())).expect("new child should work");
}

// ************************
// *** helper functions ***
// ************************

/// Create a child directory path.
fn child_dir(parent: &Path) -> PathBuf {
    let mut child = FileName(EN).fake::<String>();
    child.retain(|char| char != '.');

    parent.join(child)
}
