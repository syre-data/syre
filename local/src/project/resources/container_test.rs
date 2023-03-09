use super::*;
use crate::common::{container_file_of, container_settings_file_of};
use crate::project::container;
use crate::types::ResourceValue;
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

    assert_eq!(
        0,
        container.children.len(),
        "children should be an empty Vec"
    );

    assert_eq!(0, container.scripts.len(), "scripts should be an empty Vec");
    assert_eq!(0, container.assets.len(), "assets should be empty");
}

#[test]
fn container_default_works() {
    let container = Container::default();

    assert_eq!(
        0,
        container.children.len(),
        "children should be an empty Vec"
    );

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

    o_root.children.insert_resource(oc1.rid.clone(), oc1);
    o_root.children.insert_resource(oc2.rid.clone(), oc2);

    let path: PathBuf = FilePath(EN).fake();

    // test
    let root = o_root.duplicate().expect("could not duplicate tree");
    assert_ne!(o_root.rid, root.rid, "`ResourceId` should not match");
    assert_eq!(
        o_root.properties, root.properties,
        "properties do not match"
    );

    assert_eq!(2, root.children.len(), "incorrect children loaded");
    // @todo: Test children better.
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

// ----------------
// --- children ---
// ----------------

#[test]
fn container_register_child_should_work() {
    // setup
    let _dir = TempDir::new().expect("new `TempDir` should work");
    container::init(_dir.path()).expect("init container should work");
    let mut cont = Container::load(_dir.path()).expect("load container should work");
    let rid = ResourceId::new();

    // test
    let added = cont.register_child(rid.clone());
    assert_eq!(true, added, "child should be newly added");
    assert!(
        cont.children.contains_key(&rid),
        "child should be registered"
    );
}

#[test]
fn container_regsiter_child_should_work_if_child_already_registered() {
    // setup
    let _dir = TempDir::new().expect("new `TempDir` should work");
    container::init(_dir.path()).expect("init container should work");
    let mut cont = Container::load(_dir.path()).expect("load container should work");
    let rid = ResourceId::new();

    let added = cont.register_child(rid.clone());
    assert_eq!(true, added, "child should be newly added");
    assert!(
        cont.children.contains_key(&rid),
        "child should be registered"
    );

    // test
    let added = cont.register_child(rid.clone());
    assert_eq!(false, added, "child should not be newly added");
    assert!(
        cont.children.contains_key(&rid),
        "child should still be registered"
    );
}

#[test]
fn container_deregister_child_should_work() {
    // setup
    let _dir = TempDir::new().expect("new `TempDir` should work");
    container::init(_dir.path()).expect("init container should work");
    let mut cont = Container::load(_dir.path()).expect("load container should work");
    let rid = ResourceId::new();

    let _added = cont.register_child(rid.clone());
    assert!(
        cont.children.contains_key(&rid),
        "child should be registered"
    );

    // test
    let added = cont.deregister_child(&rid);
    assert_eq!(true, added, "child should be removed");
    assert!(
        !cont.children.contains_key(&rid),
        "child should not be registered"
    );
}

#[test]
fn container_deregister_child_should_work_if_child_not_registered() {
    // setup
    let _dir = TempDir::new().expect("new `TempDir` should work");
    container::init(_dir.path()).expect("init container should work");
    let mut cont = Container::load(_dir.path()).expect("load container should work");
    let rid = ResourceId::new();

    // test
    let added = cont.deregister_child(&rid);
    assert_eq!(false, added, "no child should be removed");
    assert!(
        !cont.children.contains_key(&rid),
        "child should not be registered"
    );
}

#[test]
fn container_get_child_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_dir = _dir.mkdir().expect("mkdir should work");

    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    let mut child = Container::load(&child_dir).expect("load child `Container` should work");

    let child_id = child.rid.clone();
    root.register_child(child_id.clone());

    root.save().expect("could not save root `Container`");
    child.save().expect("could not save child `Container`");

    drop(root);
    drop(child);

    // test
    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    assert!(
        root.children.contains_key(&child_id),
        "child `Container` not registered"
    );

    let child = root
        .get_child(&child_id)
        .expect("get child `Container` should work");

    assert_eq!(child_id, child.rid, "incorrect child `Container` retrieved");
}

#[test]
#[should_panic(expected = "NotRegistered")]
fn container_get_child_if_not_registered_should_error() {
    let container = Container::new().expect("new `Container` should work");
    container.get_child(&ResourceId::new()).unwrap();
}

#[test]
#[should_panic(expected = "MissingChild")]
fn container_get_child_if_does_not_exist_should_error() {
    // setup
    let _dir = TempDir::new().expect("new `TempDir` should work");
    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");

    let child_id = ResourceId::new();
    root.register_child(child_id.clone());

    // test
    root.get_child(&child_id).unwrap();
}

#[test]
fn container_get_children_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_dir = _dir.mkdir().expect("mkdir should work");

    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    let mut child = Container::load(&child_dir).expect("load child `Container` should work");

    let child_id = child.rid.clone();
    let child_ids = vec![child_id.clone()];

    root.register_child(child_id.clone());

    root.save().expect("could not save root `Container`");
    child.save().expect("could not save child `Container`");

    drop(root);
    drop(child);

    // test
    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    assert!(
        root.children.contains_key(&child_id),
        "child `Container` not registered"
    );

    let children = root
        .get_children()
        .expect("get child `Container` should work");

    let found_ids = children
        .iter()
        .map(|c| c.rid.clone())
        .collect::<Vec<ResourceId>>();

    assert_eq!(
        child_ids.len(),
        found_ids.len(),
        "incorrect number of child `Containers` found"
    );

    for cid in child_ids {
        assert!(
            found_ids.contains(&cid),
            "incorrect child `Container` retrieved"
        );
    }
}

#[test]
fn container_load_children_without_recursion_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_dir = _dir.mkdir().expect("make child dir should work");
    let grandchild_dir = _dir
        .children
        .get_mut(&child_dir)
        .expect("child dir should exist")
        .mkdir()
        .expect("make grandchild dir should work");

    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    let mut child = Container::load(&child_dir).expect("load child `Container` should work");
    let mut grandchild =
        Container::load(&grandchild_dir).expect("load grandchild `Container` should work");

    let child_id = child.rid.clone();
    let grandchild_id = grandchild.rid.clone();
    root.register_child(child_id.clone());
    child.register_child(grandchild_id.clone());

    root.save().expect("could not save root `Container`");
    child.save().expect("could not save child `Container`");
    grandchild
        .save()
        .expect("could not save grandchild `Container`");

    drop(root);
    drop(child);
    drop(grandchild);

    // test
    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    assert!(
        root.children.contains_key(&child_id),
        "child `Container` not registered"
    );

    root.load_children(false)
        .expect("load children should work");

    let child = root
        .children
        .get(&child_id)
        .expect("child `Container` should exist");

    let ResourceValue::Resource(child) = child else {
        panic!("child `Container` not loaded");
    };

    let child = child.lock().expect("could not lock child `Container`");
    assert_eq!(child_id, child.rid, "incorrect child `Container` found");

    let grandchild = child
        .children
        .get(&grandchild_id)
        .expect("granchild `Container` should exist");

    match grandchild {
        ResourceValue::Empty => {}
        _ => panic!("grandchild should not be loaded"),
    }
}

#[test]
fn container_load_children_with_recursion_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_dir = _dir.mkdir().expect("make child dir should work");
    let grandchild_dir = _dir
        .children
        .get_mut(&child_dir)
        .expect("child dir should exist")
        .mkdir()
        .expect("make grandchild dir should work");

    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    let mut child = Container::load(&child_dir).expect("load child `Container` should work");
    let mut grandchild =
        Container::load(&grandchild_dir).expect("load grandchild `Container` should work");

    let child_id = child.rid.clone();
    let grandchild_id = grandchild.rid.clone();
    root.register_child(child_id.clone());
    child.register_child(grandchild_id.clone());

    root.save().expect("could not save root `Container`");
    child.save().expect("could not save child `Container`");
    grandchild
        .save()
        .expect("could not save grandchild `Container`");

    drop(root);
    drop(child);
    drop(grandchild);

    // test
    let mut root = Container::load(_dir.path()).expect("load root `Container` should work");
    assert!(
        root.children.contains_key(&child_id),
        "child `Container` not registered"
    );

    root.load_children(true).expect("load children should work");

    let child = root
        .children
        .get(&child_id)
        .expect("child `Container` should exist");

    let ResourceValue::Resource(child) = child else {
        panic!("child `Container` not loaded");
    };

    let child = child.lock().expect("could not lock child `Container`");
    assert_eq!(child_id, child.rid, "incorrect child `Container` found");

    let grandchild = child
        .children
        .get(&grandchild_id)
        .expect("granchild `Container` should exist");

    let ResourceValue::Resource(grandchild) = grandchild else {
        panic!("grandchild should not be loaded");
    };

    let grandchild = grandchild
        .lock()
        .expect("could not lock grandchild `Container`");

    assert_eq!(
        grandchild_id, grandchild.rid,
        "incorrect grandchild `Container` found"
    );
}

#[test]
fn update_tree_base_paths_should_work() {
    // setup
    let mut root = Container::new().expect("could not create root `Container`");
    let mut c0 = Container::new().expect("could not create child `Container`");
    let c1 = Container::new().expect("could not create child `Container`");

    let base_path: PathBuf = FilePath(EN).fake();
    let c_path: PathBuf = FilePath(EN).fake();
    let mut c_path_exp = base_path.clone();
    c_path_exp.push(c_path.file_name().expect("could not get file name"));

    root.set_base_path(base_path.clone())
        .expect("could not set root base path");

    c0.set_base_path(c_path.clone())
        .expect("could not set child base path");

    let c0_rid = c0.rid.clone();
    let c1_rid = c1.rid.clone();
    root.children.insert_resource(c0.rid.clone(), c0);
    root.children.insert_resource(c1.rid.clone(), c1);

    let c0 = root
        .children
        .get_resource(&c0_rid)
        .expect("could not get child `Container`")
        .expect("child `Container` not loaded");

    let c1 = root
        .children
        .get_resource(&c1_rid)
        .expect("could not get child `Container`")
        .expect("child `Container` not loaded");

    // test
    root.update_tree_base_paths()
        .expect("could not update paths");

    assert_eq!(
        base_path,
        root.base_path().expect("could not get root base path"),
        "incorrect base path"
    );

    let c0 = c0.lock().expect("could not lock child `Container`");
    let c1 = c1.lock().expect("could not lock child `Container`");
    assert_eq!(
        c_path_exp,
        c0.base_path().expect("could not get child base path"),
        "incorrect base path"
    );

    assert!(c1.base_path().is_err(), "base path should not be set");
}

// @todo
/* Functions possibly no longer needed.

#[test]
fn container_child_path_should_work() {
    // setup
    // root
    let mut _dir = TempDir::new().expect("n `TempDir` should work");
    container::init(_dir.path()).expect("init root container should work");
    let mut c_root = Container::load(_dir.path()).expect("load root container should work");

    // child
    let c_path = _dir.mkdir().expect("make child directory should work");
    let cid = container::init(&c_path).expect("init child should work");
    c_root.register_child(cid.clone());

    // test
    let found = c_root.child_path(&cid).expect("child path should work");
    assert_eq!(c_path, found, "correct path should be found");
}

#[test]
#[should_panic(expected = "NotRegistered")]
fn container_child_test_with_invalid_resource_id_should_error() {
    // setup
    // root
    let mut _dir = TempDir::new().expect("n `TempDir` should work");
    container::init(_dir.path()).expect("init root container should work");
    let c_root = Container::load(_dir.path()).expect("load root container should work");

    // child - not registered
    let c_path = _dir.mkdir().expect("make child directory should work");
    let cid = container::init(&c_path).expect("init child should work");

    // test
    c_root.child_path(&cid).unwrap();
}

#[test]
#[should_panic(expected = "MissingChild")]
fn container_child_test_with_missing_child_directory_should_error() {
    // setup
    // root
    let mut _dir = TempDir::new().expect("n `TempDir` should work");
    container::init(_dir.path()).expect("init root container should work");
    let mut c_root = Container::load(_dir.path()).expect("load root container should work");

    let cid = ResourceId::new();
    c_root.register_child(cid.clone());

    // test
    c_root.child_path(&cid).unwrap();
}

#[test]
fn container_children_paths_should_work() {
    // setup
    // root
    let mut _dir = TempDir::new().expect("n `TempDir` should work");
    container::init(_dir.path()).expect("init root container should work");
    let mut c_root = Container::load(_dir.path()).expect("load root container should work");

    // registered child
    let r_child = _dir.mkdir().expect("make child directory should work");
    let cid = container::init(&r_child).expect("init child should work");
    c_root.register_child(cid.clone());

    // unregistered child
    let u_child = _dir.mkdir().expect("make child directory should work");
    let _uid = container::init(&u_child).expect("init child should work");

    c_root.save().expect("save should work");

    // test
    let paths = c_root.children_paths().expect("children paths should work");
    assert!(
        paths.contains(&r_child),
        "registered child should be contained"
    );

    assert!(
        !paths.contains(&u_child),
        "unregistered child should not be contained"
    );
}

*/

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

    // child
    let cid = ResourceId::new();
    container.register_child(cid);

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
