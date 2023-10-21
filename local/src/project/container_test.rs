use super::*;
use crate::common;
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
fn builder_init_no_assets_no_recurse_on_non_resource_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let root = _dir.path().join("container");
    fs::create_dir(&root).expect("create directory should work");

    // test
    let builder = InitOptions::init();
    builder.build(root.as_path()).unwrap();

    // check .thot folder created
    assert!(
        common::thot_dir_of(&root).exists(),
        ".thot folder should exist"
    );

    // check files exist and are empty
    assert!(
        common::container_file_of(&root).exists(),
        "container file should exist"
    );

    assert!(
        common::container_settings_file_of(&root).exists(),
        "container settings file should exist"
    );

    assert!(
        common::assets_file_of(&root).exists(),
        "assets file should exist"
    );

    // ensure container is correct
    let container = Container::load_from(&root).unwrap();
    assert_eq!(
        root.file_name().unwrap().to_str().unwrap(),
        container.properties.name,
        "container's name should match folder"
    );

    assert!(
        container.assets.is_empty(),
        "container should not have assets"
    );
}

#[test]
fn builder_init_should_return_resource_id_if_already_a_container() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");

    let builder = InitOptions::init();
    let rid = builder.build(_dir.path()).expect("init should work");

    // test
    let found_rid = builder
        .build(_dir.path())
        .expect("init should return old resource id");

    assert_eq!(rid, found_rid, "resource ids should match");
}

#[test]
#[should_panic]
fn builder_init_if_folder_is_a_thot_resource_but_not_a_container_should_error() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    fs::create_dir(common::thot_dir_of(_dir.path())).expect("creating thot directory should work");

    // test
    let builder = InitOptions::init();
    builder.build(_dir.path()).expect("init should work");
}

#[test]
#[should_panic(expected = "NotADirectory")]
fn builder_init_should_error_if_folder_does_not_exist() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let path = _dir.path().join("root");

    // test
    let builder = InitOptions::init();
    builder.build(path.as_path()).unwrap();
}

#[test]
fn builder_new_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");

    // test
    let c_path = _dir.path().join("container");
    let builder = InitOptions::new();
    builder.build(c_path.as_path()).expect("new should work");
}

#[test]
fn builder_new_with_properties_should_work() {
    // setup
    let _dir = TempDir::new().expect("`TempDir::new` should work");
    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    let kind: Vec<String> = Words(EN, 3..5).fake();
    let kind = kind.join(" ");
    let mut properties = ContainerProperties::new(name);
    properties.kind = Some(kind);

    // test
    let mut builder = InitOptions::new();
    builder.properties(properties.clone());
    builder.build(_dir.path()).expect("`init_from` should work");

    let c = Container::load_from(_dir.path()).expect("could not load `Container`");
    assert_eq!(properties.kind, c.properties.kind, "`kind`s do not match");
    assert_eq!(
        _dir.path().file_name().unwrap().to_str().unwrap(),
        c.properties.name,
        "name should be changed to folder"
    );
}
