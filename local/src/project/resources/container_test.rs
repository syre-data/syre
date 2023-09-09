use super::*;
use crate::common::{container_file_of, container_settings_file_of};
use crate::project::container;
use dev_utils::fs::TempDir;
use dev_utils::path::resource_path::resource_path;
use fake::faker::filesystem::raw::FilePath;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::Fake;
use std::path::Path;
use thot_core::project::{
    Asset as CoreAsset, Container as CoreContainer, RunParameters, ScriptAssociation,
};
use thot_core::types::{ResourceId, ResourcePath};

// *****************
// *** Container ***
// *****************

// --------------------------
// --- script assocations ---
// --------------------------

#[test]
fn container_contains_script_association_should_work() {
    // setup
    let dir = TempDir::new().unwrap();
    let mut container = Container::new(dir.path());
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
    let dir = TempDir::new().unwrap();
    let mut container = Container::new(dir.path());
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
    let dir = TempDir::new().unwrap();
    let mut container = Container::new(dir.path());
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
    let dir = TempDir::new().unwrap();
    let mut container = Container::new(dir.path());
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
    let dir = TempDir::new().unwrap();
    let mut container = Container::new(dir.path());
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

// -------------
// --- serde ---
// -------------

#[test]
fn container_serde_serialization_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    container::init(_dir.path()).expect("init container should work");
    let mut container = Container::load_from(_dir.path()).expect("load container should work");

    // asset
    let a_path = _dir.mkfile().expect("mkfile should work");
    let asset = Asset::new(ResourcePath::new(a_path).unwrap());
    container.insert_asset(asset);

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
