use super::*;
use dev_utils::fs::TempDir;
use dev_utils::path::resource_path::resource_path;
use std::fs;
use thot_core::db::StandardSearchFilter as StdFilter;
use thot_core::types::ResourceId;
use thot_local::loader::{
    container::Loader as ContainerLoader, tree::Loader as ContainerTreeLoader,
};
use thot_local::project::resources::{
    Asset as LocalAsset, Container as LocalContainer, Project as LocalProject,
    Scripts as LocalScripts,
};

#[test]
fn insert_project_should_work() {
    // setup
    let dir = TempDir::new().expect("new `TempDir` should work");
    let project = LocalProject::load_from(dir.path()).expect("load `Project` should work");
    let pid = project.rid.clone();

    let mut db = Datastore::new();

    // test
    db.insert_project(project)
        .expect("`insert_project` should work");

    assert!(db.projects.contains_key(&pid), "`Project` not inserted");
    assert!(
        db.project_paths.contains_key(dir.path()),
        "`Project` path not inserted",
    );

    let project = db.projects.get(&pid).expect("`Project` not loaded");
    let rid = db
        .project_paths
        .get(dir.path())
        .expect("`Project` path not loaded");

    assert_eq!(&pid, &project.rid, "incorrect `Project` inserted");
    assert_eq!(&pid, rid, "incorrect path map");
}

#[test]
fn update_project_path_should_work() {
    // setup
    let dir = tempfile::tempdir().unwrap();
    let prj = dir.path().join("project");
    let root = prj.join("data");
    let child = root.join("child");
    let root_asset = root.join("asset");
    let child_asset = child.join("asset");

    fs::create_dir(&prj).unwrap();
    fs::create_dir(&root).unwrap();
    fs::create_dir(&child).unwrap();
    fs::write(&root_asset, "").unwrap();
    fs::write(&child_asset, "").unwrap();

    let mut project = LocalProject::new(dir.path().join(prj)).unwrap();
    project.data_root = Some(root.clone());
    project.save().unwrap();
    let pid = project.rid.clone();
    let project_path = project.base_path().to_path_buf();

    let mut root = LocalContainer::new(project.data_root_path().unwrap());
    let asset = LocalAsset::new(ResourcePath::new("asset").unwrap()).unwrap();
    root.insert_asset(asset);
    root.save().unwrap();

    let mut child = LocalContainer::new(root.base_path().join(child));
    let asset = LocalAsset::new(ResourcePath::new("asset").unwrap()).unwrap();
    child.insert_asset(asset);
    child.save().unwrap();

    let mut graph = ResourceTree::new(root);
    graph.insert(graph.root().clone(), child).unwrap();

    let mut db = Datastore::new();
    db.insert_project(project).unwrap();
    db.insert_project_graph(pid.clone(), graph).unwrap();

    // test
    let mut prj = project_path.clone();
    prj.set_file_name("new");

    let root = prj.join("data");
    let child = root.join("child");
    let root_asset = root.join("asset");
    let child_asset = child.join("asset");

    fs::rename(project_path, &prj).unwrap();
    db.update_project_path(&pid, &prj).unwrap();

    assert_eq!(&pid, db.get_path_project_canonical(&prj).unwrap().unwrap());
    assert!(db.get_path_container_canonical(&root).unwrap().is_some());
    assert!(db.get_path_container_canonical(&child).unwrap().is_some());
    assert!(db
        .get_path_asset_id_canonical(&root_asset)
        .unwrap()
        .is_some());

    assert!(db
        .get_path_asset_id_canonical(&child_asset)
        .unwrap()
        .is_some());
}

#[test]
fn insert_project_graph_should_work() {
    // setup
    let pid = ResourceId::new();
    let mut dir = TempDir::new().unwrap();
    let child_dir = dir.mkdir().unwrap();

    let mut root = ContainerLoader::load(dir.path()).unwrap();
    let mut child = ContainerLoader::load(&child_dir).unwrap();

    let a0 = LocalAsset::new(resource_path(Some("py"))).unwrap();
    let a1 = LocalAsset::new(resource_path(Some("py"))).unwrap();

    let c_root_rid = root.rid.clone();
    let c_child_rid = child.rid.clone();
    let aids = vec![a0.rid.clone(), a1.rid.clone()];

    root.insert_asset(a0).unwrap();

    child.insert_asset(a1).unwrap();

    root.save().unwrap();
    child.save().unwrap();

    drop(root);
    drop(child);

    let graph = ContainerTreeLoader::load(dir.path()).unwrap();
    let mut db = Datastore::new();

    // test
    db.insert_project_graph_canonical(pid.clone(), graph)
        .unwrap();

    // containers
    assert!(db.graphs.contains_key(&pid), "`Project` not inserted");

    assert_eq!(
        &pid,
        db.container_projects
            .get(&c_root_rid)
            .expect("`Container` not registered with project"),
        "root `Container` not inserted"
    );

    assert_eq!(
        &pid,
        db.container_projects
            .get(&c_child_rid)
            .expect("`Container` not registered with project"),
        "child `Container` not inserted"
    );

    // assets
    for rid in aids {
        assert!(db.asset_containers.contains_key(&rid), "asset not inserted");
    }
}

#[test]
fn get_container_should_work() {
    // setup
    let dir = TempDir::new().unwrap();
    let mut db = Datastore::new();
    let container = LocalContainer::new(dir.path());
    let rid = container.rid.clone();
    let graph = ResourceTree::new(container);

    db.insert_project_graph_canonical(ResourceId::new(), graph)
        .unwrap();

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
    let dir = TempDir::new().unwrap();
    let mut db = Datastore::new();
    let mut container = LocalContainer::new(dir.path());
    let cid = container.rid.clone();

    let asset = LocalAsset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    let aid = asset.rid.clone();

    container.insert_asset(asset);

    let graph = ResourceTree::new(container);
    db.insert_project_graph_canonical(ResourceId::new(), graph)
        .unwrap();

    // test
    let Some(found) = db.get_asset_container(&aid) else {
        panic!("container should have been found");
    };

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
    let mut dir = TempDir::new().expect("new `TempDir` should work");
    let child_1_dir = dir.mkdir().expect("mkdir should work");
    let child_2_dir = dir.mkdir().expect("mkdir should work");

    let mut root = LocalContainer::new(dir.path());
    let mut child_1 = LocalContainer::new(&child_1_dir);
    let child_2 = LocalContainer::new(&child_2_dir);

    let root_rid = root.rid.clone();
    let child_1_rid = child_1.rid.clone();
    let child_2_rid = child_2.rid.clone();

    let find_kind = Some("find".to_string());
    root.properties.kind = find_kind.clone();
    child_1.properties.kind = find_kind.clone();

    let mut db = Datastore::new();

    let mut graph = ResourceTree::new(root);
    graph.insert(root_rid.clone(), child_1).unwrap();
    graph.insert(root_rid.clone(), child_2).unwrap();
    db.insert_project_graph_canonical(ResourceId::new(), graph)
        .unwrap();

    let mut find_filter = StdFilter::default();
    find_filter.kind = Some(find_kind);

    // test
    // root not loaded
    let found = db.find_containers(&ResourceId::new(), StdFilter::default());
    assert_eq!(0, found.len(), "no `Container`s should be found");

    // find from root
    let found = db.find_containers(&root_rid, find_filter.clone());
    let found_ids = found
        .iter()
        .map(|c| c.rid.clone())
        .collect::<Vec<ResourceId>>();

    assert!(
        found_ids.contains(&root_rid),
        "root `Container` should be found"
    );

    assert!(
        found_ids.contains(&child_1_rid),
        "child `Container` should be found"
    );

    assert!(
        !found_ids.contains(&child_2_rid),
        "child `Container` should not be found"
    );

    // find from child
    let found = db.find_containers(&child_1_rid, find_filter.clone());
    let found_ids = found
        .iter()
        .map(|c| c.rid.clone())
        .collect::<Vec<ResourceId>>();

    assert!(
        !found_ids.contains(&root_rid),
        "root `Container` should not be found"
    );

    assert!(
        found_ids.contains(&child_1_rid),
        "child `Container` should be found"
    );

    assert!(
        !found_ids.contains(&child_2_rid),
        "child `Container` should not be found"
    );
}

#[test]
fn find_assets_should_work() {
    // setup
    let mut _dir = TempDir::new().expect("new `TempDir` should work");
    let child_dir = _dir.mkdir().expect("mkdir should work");

    let mut root = LocalContainer::new(_dir.path());
    let mut child = LocalContainer::new(&child_dir);

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

    let mut db = Datastore::new();
    let mut graph = ResourceTree::new(root);
    graph.insert(root_rid.clone(), child).unwrap();
    db.insert_project_graph_canonical(ResourceId::new(), graph)
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
    let mut _dir = TempDir::new().unwrap();
    let project = LocalProject::load_from(_dir.path()).unwrap();
    let pid = project.rid.clone();

    let db = Datastore::new();

    // test
    todo!();
}

#[test]
fn remove_project_script_should_work() {
    // setup
    let _dir = TempDir::new().expect("could not create temporary directory");

    let pid = ResourceId::new();

    let mut scripts = LocalScripts::load_from(_dir.path()).expect("could not load `Scripts`");
    let script = CoreScript::new(resource_path(Some("py"))).expect("could not create `Script`");
    let sid = script.rid.clone();

    scripts
        .insert_script(script)
        .expect("could not insert `Script`");

    // add other script that is not to be removed
    let other_script =
        CoreScript::new(resource_path(Some("py"))).expect("could not create `Script`");
    let other_sid = other_script.rid.clone();

    scripts
        .insert_script(other_script)
        .expect("could not insert other `Script`");

    let mut store = Datastore::new();

    store.insert_project_scripts(pid.clone(), scripts);

    // test
    store
        .remove_project_script(&pid, &sid)
        .expect("could not remove `Script`");

    let scripts = store
        .get_project_scripts(&pid)
        .expect("could not get `Scripts`");

    assert!(
        !scripts.contains_key(&sid),
        "removed script should not be there"
    );

    assert!(
        !store.script_projects.contains_key(&sid),
        "project map for removed script should not exist"
    );

    assert!(
        scripts.contains_key(&other_sid),
        "non removed script should be there"
    );

    assert!(
        store.script_projects.contains_key(&other_sid),
        "project map for not removed script should exist"
    );
}
