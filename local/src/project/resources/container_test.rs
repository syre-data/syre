use super::*;
use dev_utils::fs::TempDir;
use syre_core::project::{RunParameters, ScriptAssociation};
use syre_core::types::ResourceId;

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
    let init = container.set_script_association(assoc.clone());
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
    let sec = container.set_script_association(assoc.clone());
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
