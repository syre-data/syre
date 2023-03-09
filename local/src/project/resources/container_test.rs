use super::*;
use crate::common::{container_file_of, container_settings_file_of};
use crate::project::container;
use dev_utils::fs::TempDir;
use dev_utils::path::resource_path::resource_path;
use fake::faker::filesystem::raw::FilePath;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;
use settings_manager::types::Priority as SettingsPriority;
use std::path::Path;
use thot_core::project::{
    Asset as CoreAsset, Container as CoreContainer, RunParameters, ScriptAssociation,
};
use thot_core::types::ResourceId;

// *****************
// *** Container ***
// *****************

#[test]
fn container_new_works() {
    let container = Container::new().expect("new should work");

    assert_eq!(0, container.scripts.len(), "scripts should be an empty Vec");
    assert_eq!(0, container.assets.len(), "assets should be empty");
}

#[test]
fn container_default_works() {
    let container = Container::default();

    assert_eq!(0, container.scripts.len(), "scripts should be an empty Vec");
    assert_eq!(0, container.assets.len(), "assets should be empty");
}

#[test]
fn container_priority_should_be_correct() {
    let container = Container::default();

    assert_eq!(
        SettingsPriority::Local,
        container.priority(),
        "priority should be correct"
    );
}

#[test]
fn container_rel_path_should_be_correct() {
    let path = Container::rel_path().expect("rel_path should work");

    assert_eq!(
        container_file_of(Path::new("")),
        path,
        "rel_path should be correct"
    );
}

#[test]
fn duplicate_with_no_children_should_work() {
    // setup
    let mut o_root = Container::new().expect("could not create `Container`");
    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    o_root.properties.name = Some(name);

    let path: PathBuf = FilePath(EN).fake();
    o_root.set_base_path(path).expect("could not set base path");

    // test
    let root = o_root.duplicate().expect("could not duplicate tree");

    assert_ne!(o_root.rid, root.rid, "`ResourceId` should not match");
    assert_eq!(
        o_root.properties, root.properties,
        "properties do not match"
    );

    assert!(root.base_path().is_err(), "path should not be set");
}

#[test]
fn duplicate_should_work() {
    // setup
    let mut o_root = Container::new().expect("could not create root `Container`");
    let mut oc1 = Container::new().expect("could not create child `Container`");
    let mut oc2 = Container::new().expect("could not create child `Container`");

    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    o_root.properties.name = Some(name);

    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    oc1.properties.name = Some(name);

    let name: Vec<String> = Words(EN, 3..5).fake();
    let name = name.join(" ");
    oc2.properties.name = Some(name);

    let path: PathBuf = FilePath(EN).fake();

    // test
    let root = o_root.duplicate().expect("could not duplicate tree");
    assert_ne!(o_root.rid, root.rid, "`ResourceId` should not match");
    assert_eq!(
        o_root.properties, root.properties,
        "properties do not match"
    );
}

// -------------
// --- asset ---
// -------------

#[test]
fn container_new_asset_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let mut cont = Container::new().expect("new `Container` should work");
    cont.set_base_path(_dir.path().to_path_buf());

    // test
    let file = _dir
        .mkfile_with_extension("py")
        .expect("mkfile should work");

    let rid = cont.new_asset(&file).expect("new asset should work");
    assert!(cont.assets.contains_key(&rid), "asset was not inserted");
}

#[test]
#[should_panic(expected = "PathNotSet")]
fn container_new_asset_without_base_path_should_error() {
    todo!()
}

#[test]
fn container_remove_asset_should_work() {
    // setup
    let mut cont = Container::new().expect("new `Container` should work");

    let file_path = resource_path(Some("py"));
    let asset = CoreAsset::new(file_path);
    let rid = asset.rid.clone();

    cont.insert_asset(asset).expect("insert asset should work");

    // test
    let res = cont.remove_asset(&rid);
    assert!(res.is_some(), "asset should have been registered");
    assert!(
        !cont.assets.contains_key(&rid),
        "container should not have asset registered"
    );
}

#[test]
fn container_remove_asset_should_do_nothing_if_asset_is_not_present() {
    // setup
    let _dir = TempDir::new().expect("new `TempDir` should work");
    container::init(_dir.path()).expect("init container should work");
    let mut cont = Container::load(_dir.path()).expect("load container should work");
    let rid = ResourceId::new();

    // test
    let res = cont.remove_asset(&rid);
    assert_eq!(None, res, "asset should not be present");
    assert!(
        !cont.assets.contains_key(&rid),
        "container should not be present"
    );
}

// --------------------------
// --- script assocations ---
// --------------------------

#[test]
fn container_contains_script_association_should_work() {
    // setup
    let mut container = Container::new().expect("creating container should work");
    let sid = ResourceId::new();
    let assoc = ScriptAssociation::new(sid.clone());
    container.scripts.insert(sid.clone(), assoc.into());

    // test
    assert!(
        container.contains_script_association(&sid),
        "container should have script association"
    );
    assert_eq!(
        false,
        container.contains_script_association(&ResourceId::new()),
        "container should not have association with script"
    );
}

#[test]
fn container_add_script_association_should_work() {
    // setup
    let mut container = Container::new().expect("creating container should work");
    let sid = ResourceId::new();
    let assoc = ScriptAssociation::new(sid.clone());

    // test
    container
        .add_script_association(assoc)
        .expect("add association should work");
    assert!(
        container.contains_script_association(&sid),
        "container should contain association"
    );
}

#[test]
#[should_panic(expected = "AlreadyExists")]
fn container_add_script_association_if_already_exists_should_error() {
    // setup
    let mut container = Container::new().expect("creating container should work");
    let sid = ResourceId::new();
    let assoc = ScriptAssociation::new(sid.clone());
    container
        .add_script_association(assoc.clone())
        .expect("add association should work");

    // test
    container.add_script_association(assoc).unwrap();
}

#[test]
fn container_set_script_association_should_work() {
    // setup
    let mut container = Container::new().expect("creating container should work");
    let sid = ResourceId::new();
    let mut assoc = ScriptAssociation::new(sid.clone());

    // test
    // initial
    let init = container
        .set_script_association(assoc.clone())
        .expect("intial set association should work");

    let found = container.scripts.get(&sid);
    assert!(found.is_some(), "association should be added");

    let found = found.unwrap();
    assert!(init, "initial association add should return true");
    assert_eq!(
        &assoc.priority, &found.priority,
        "association should be set"
    );

    // second
    assoc.priority = 1;
    let sec = container
        .set_script_association(assoc.clone())
        .expect("second set association should work");
    let found = container.scripts.get(&sid);
    assert!(found.is_some(), "association should still exist");

    let found = found.unwrap();
    assert_eq!(false, sec, "second associaiton set should return false");
    assert_eq!(
        &assoc.priority, &found.priority,
        "association should be updated"
    );
}

#[test]
fn container_remove_script_association_should_work() {
    // setup
    let mut container = Container::new().expect("creating container should work");
    let sid = ResourceId::new();
    let params = RunParameters::new();
    container.scripts.insert(sid.clone(), params);

    // test
    // first
    let init = container.remove_script_association(&sid);
    assert_eq!(
        false,
        container.contains_script_association(&sid),
        "association should no longer exist"
    );
    assert!(init, "remove should return true");

    // second
    let sec = container.remove_script_association(&sid);
    assert_eq!(false, sec, "remove should return false");
}

// ----------------------
// --- Core Container ---
// ----------------------

#[test]
fn container_from_core_should_work() {
    // setup
    let core_container = CoreContainer::default();
    let rid = core_container.rid.clone();

    // test
    let container: Container = Container::from(core_container);
    assert_eq!(rid, container.rid, "resource ids should match");
}

#[test]
fn container_into_core_should_work() {
    // setup
    let container = Container::new().expect("new should work");
    let rid = container.rid.clone();

    // test
    let core: CoreContainer = container.into();
    assert_eq!(rid, core.rid, "resource ids should match");
}

// -------------
// --- serde ---
// -------------

#[test]
fn container_serde_serialization_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    container::init(_dir.path()).expect("init container should work");
    let mut container = Container::load(_dir.path()).expect("load container should work");
    container
        .set_base_path(_dir.path().to_path_buf())
        .expect("set base path should work");

    // asset
    let a_path = _dir.mkfile().expect("mkfile should work");
    container
        .new_asset(&a_path)
        .expect("register asset should work");

    // script association
    let sid = ResourceId::new();
    let assoc = ScriptAssociation::new(sid.clone());
    container
        .add_script_association(assoc.clone())
        .expect("adding script association should work");

    // test
    let _json = serde_json::to_string(&container).expect("serialization should work");
    println!("{:?}", _json);
}

#[test]
fn container_serde_deserialization_should_work() {
    // setup
    let json = r#"{
        "rid":"761474b7-0b71-47de-8751-f28d44958100",
        "properties":{
            "created":"2022-10-19T20:45:38.194421359Z",
            "creator":{
                "User":null
            },
            "permissions":{
                
            },
            "name":null,
            "kind":null,
            "description":null,
            "tags":[
                
            ],
            "metadata":{
                
            }
        },
        "children":[
                "d228cc54-bba2-40cb-a77b-50562415ff52"
        ],
        "assets":[
                "956581ec-3a50-4399-9d2c-e54ed6776a86"
        ],
        "scripts":[
            {
                "script":"9983bfd5-8d85-41ea-af51-d16ac0e188e7",
                "autorun":true,
                "priority":0
            }
        ]
    }"#;

    // test
    let _container: Container = serde_json::from_str(&json).expect("deserialize should work");
}

// **************************
// *** Container Settings ***
// **************************

#[test]
fn container_settings_new_should_work() {
    let sets = ContainerSettings::new();
    assert_eq!(0, sets.permissions.len(), "permissions should be empty");
}

#[test]
fn container_settings_relative_path_is_correct() {
    let path = ContainerSettings::rel_path().expect("relative path should work");
    assert_eq!(
        container_settings_file_of(Path::new("")),
        path,
        "realtive path should be correct"
    );
}
