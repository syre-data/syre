use super::*;
use dev_utils::fs::TempDir;
use dev_utils::path::resource_path::resource_path;
use fake::faker::filesystem::raw::DirPath;
use fake::locales::EN;
use fake::Fake;
use std::sync::{Arc, Mutex};
use thot_core::db::StandardSearchFilter as StdFilter;
use thot_core::types::ResourceId;
use thot_local::project::resources::{
    Asset as LocalAsset, Container as LocalContainer, Project as LocalProject,
};

#[test]
fn new_should_work() {
    Datastore::new();
}

#[test]
fn insert_project_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let project = LocalProject::load(_dir.path()).expect("load `Project` should work");
    let pid = project.rid.clone();

    let mut db = Datastore::new();

    // test
    db.insert_project(project)
        .expect("`insert_project` should work");

    assert!(db.projects.contains_key(&pid), "`Project` not inserted");
    assert!(
        db.project_paths.contains_key(_dir.path()),
        "`Project` path not inserted",
    );

    let project = db.projects.get(&pid).expect("`Project` not loaded");
    let rid = db
        .project_paths
        .get(_dir.path())
        .expect("`Project` path not loaded");

    assert_eq!(&pid, &project.rid, "incorrect `Project` inserted");
    assert_eq!(&pid, rid, "incorrect path map");
}

#[test]
fn insert_container_tree_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_dir = _dir.mkdir().expect("mkdir should work");

    let mut root = LocalContainer::load(_dir.path()).expect("load `Container` should work");
    let mut child = LocalContainer::load(&child_dir).expect("load child `Container` should work");
    root.register_child(child.rid.clone());

    let a0 = LocalAsset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    let a1 = LocalAsset::new(resource_path(Some("py"))).expect("new `Asset` should work");

    let c_root_rid = root.rid.clone();
    let c_child_rid = child.rid.clone();
    let aids = vec![a0.rid.clone(), a1.rid.clone()];

    root.assets
        .insert_asset(a0)
        .expect("could not insert `Asset` into root `Container`");

    child
        .insert_asset(a1)
        .expect("could not insert `Asset` into child `Container`");

    root.save().expect("could not save root `Container`");
    child.save().expect("could not save child `Container`");
    drop(child);

    root.load_children(true)
        .expect("could not load `Container` children");

    let root = Arc::new(Mutex::new(root));
    let mut db = Datastore::new();

    // test
    db.insert_container_tree(root.clone())
        .expect("load container tree should work");

    // containers
    assert!(
        db.containers.contains_key(&c_root_rid),
        "root `Container` not inserted"
    );

    assert!(
        db.containers.contains_key(&c_child_rid),
        "child `Container` not inserted"
    );

    // assets
    for rid in aids {
        assert!(db.assets.contains_key(&rid), "asset not inserted");
    }

    // second insert
    db.insert_container_tree(root)
        .expect("load container tree should work");
}

#[test]
fn insert_container_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let mut container = LocalContainer::load(_dir.path()).expect("load `Container` should work");
    let asset = LocalAsset::new(resource_path(Some("py"))).expect("new `Asset` should work");

    let cid = container.rid.clone();
    let aid = asset.rid.clone();

    container
        .assets
        .insert_asset(asset)
        .expect("could not insert `Asset`s");

    container.save().expect("save `Container` should work");
    let mut db = Datastore::new();

    // test
    db.insert_container(container)
        .expect("load container should work");

    assert!(db.containers.contains_key(&cid), "container not inserted");
    assert!(db.assets.contains_key(&aid), "asset not inserted");
}

#[test]
fn get_container_should_work() {
    // setup
    let mut db = Datastore::new();
    let container = LocalContainer::new().expect("new `Container` should work");
    let rid = container.rid.clone();

    db.containers
        .insert(container.rid.clone(), Arc::new(Mutex::new(container)));

    // test
    let found = db.get_container(&rid);
    assert!(found.is_some(), "container should be found");

    // find non-existant
    let found = db.get_container(&ResourceId::new());
    assert!(found.is_none(), "no container should be found");
}

#[test]
fn get_asset_container_should_work() {
    // setup
    let mut db = Datastore::new();
    let mut container = LocalContainer::new().expect("new `Container` should work");

    let cid = container.rid.clone();
    let c_path = PathBuf::from(DirPath(EN).fake::<String>());
    container
        .set_base_path(c_path)
        .expect("could not set `Container` `base_path`");

    let asset = LocalAsset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    let aid = asset.rid.clone();

    container.assets.insert(asset.rid.clone(), asset);
    db.insert_container(container)
        .expect("could not insert `Container`");

    // test
    let Some(found) = db.get_asset_container(&aid) else {
        panic!("container should have been found");
    };

    let found = found.lock().expect("could not lock container");
    assert_eq!(cid, found.rid, "incorrect container found");

    // get non-existant
    let found = db.get_asset_container(&ResourceId::new());
    assert!(found.is_none(), "container should not be found");
}

#[test]
fn update_container_should_work() {
    todo!();
}

#[test]
fn find_containers_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_1_dir = _dir.mkdir().expect("mkdir should work");
    let child_2_dir = _dir.mkdir().expect("mkdir should work");

    let mut root = LocalContainer::load(_dir.path()).expect("load `Container` should work");
    let mut child_1 =
        LocalContainer::load(&child_1_dir).expect("load child `Container` should work");

    let mut child_2 =
        LocalContainer::load(&child_2_dir).expect("load child `Container` should work");

    root.register_child(child_1.rid.clone());
    root.register_child(child_2.rid.clone());

    let root_rid = root.rid.clone();
    let child_1_rid = child_1.rid.clone();
    let child_2_rid = child_2.rid.clone();

    let find_kind = Some("find".to_string());
    root.properties.kind = find_kind.clone();
    child_1.properties.kind = find_kind.clone();

    root.save().expect("could not save root `Container`");
    child_1.save().expect("could not save child `Container`");
    child_2.save().expect("could not save child `Container`");
    drop(child_1);
    drop(child_2);

    root.load_children(true)
        .expect("could not load `Container` children");

    let root = Arc::new(Mutex::new(root));
    let mut db = Datastore::new();

    db.insert_container_tree(root.clone())
        .expect("load container tree should work");

    let mut find_filter = StdFilter::default();
    find_filter.kind = Some(find_kind);

    // test
    // root not loaded
    let found = db.find_containers(&ResourceId::new(), StdFilter::default());
    assert_eq!(0, found.len(), "no `Container`s should be found");

    // find from root
    let found = db.find_containers(&root_rid, find_filter.clone());
    assert!(
        found.contains_key(&root_rid),
        "root `Container` should be found"
    );

    assert!(
        found.contains_key(&child_1_rid),
        "child `Container` should be found"
    );

    assert!(
        !found.contains_key(&child_2_rid),
        "child `Container` should not be found"
    );

    // find from child
    let found = db.find_containers(&child_1_rid, find_filter.clone());
    assert!(
        !found.contains_key(&root_rid),
        "root `Container` should not be found"
    );

    assert!(
        found.contains_key(&child_1_rid),
        "child `Container` should be found"
    );

    assert!(
        !found.contains_key(&child_2_rid),
        "child `Container` should not be found"
    );
}

#[test]
fn find_assets_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_dir = _dir.mkdir().expect("mkdir should work");

    let mut root = LocalContainer::load(_dir.path()).expect("load `Container` should work");
    let mut child = LocalContainer::load(&child_dir).expect("load child `Container` should work");
    root.register_child(child.rid.clone());

    let mut a0 = LocalAsset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    let mut a1 = LocalAsset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    let find_kind = Some("find".to_string());

    let a0_name = Some("A0".to_string());
    a0.properties.name = a0_name.clone();
    a0.properties.kind = find_kind.clone();
    a1.properties.kind = find_kind.clone();

    let root_rid = root.rid.clone();
    let child_rid = child.rid.clone();
    let a0_rid = a0.rid.clone();
    let a1_rid = a1.rid.clone();

    root.insert_asset(a0)
        .expect("could not insert `Asset` into root `Container`");

    child
        .insert_asset(a1)
        .expect("could not insert `Asset` into child `Container`");

    root.save().expect("could not save root `Container`");
    child.save().expect("could not save child `Container`");
    drop(child);

    root.load_children(true)
        .expect("could not load `Container` children");

    let root = Arc::new(Mutex::new(root));
    let mut db = Datastore::new();

    db.insert_container_tree(root.clone())
        .expect("load container tree should work");

    let mut kind_filter = StdFilter::default();
    kind_filter.kind = Some(find_kind);

    // test
    // root container not loaded
    let found = db.find_assets(&ResourceId::new(), StdFilter::default());
    assert_eq!(0, found.len(), "no `Asset`s should be found");

    // find from root
    let kind_found = db.find_assets(&root_rid, kind_filter.clone());
    let kind_found = kind_found
        .into_iter()
        .map(|asset| asset.rid)
        .collect::<Vec<ResourceId>>();

    assert!(kind_found.contains(&a0_rid), "`Asset` should be found");
    assert!(kind_found.contains(&a1_rid), "`Asset` should be found");

    let mut name_filter = StdFilter::default();
    name_filter.name = Some(a0_name);
    let name_found = db.find_assets(&root_rid, name_filter);
    let name_found = name_found
        .into_iter()
        .map(|asset| asset.rid)
        .collect::<Vec<ResourceId>>();

    assert!(
        name_found.contains(&a0_rid),
        "named `Asset` should be found"
    );

    assert!(
        !name_found.contains(&a1_rid),
        "unnamed `Asset` should not be found"
    );

    // find from child
    let kind_found = db.find_assets(&child_rid, kind_filter.clone());
    let kind_found = kind_found
        .into_iter()
        .map(|asset| asset.rid)
        .collect::<Vec<ResourceId>>();

    assert!(
        !kind_found.contains(&a0_rid),
        "root `Asset` should not be found"
    );
    assert!(
        kind_found.contains(&a1_rid),
        "child `Asset` should be found"
    );
}

#[test]
fn insert_project_scripts_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let project = LocalProject::load(_dir.path()).expect("load `Project` should work");
    let pid = project.rid.clone();

    let mut db = Datastore::new();

    // test
    todo!();
}
