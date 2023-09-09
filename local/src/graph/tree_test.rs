use super::*;
use crate::project::container;
use dev_utils::fs::TempDir;

#[test]
fn container_tree_load_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create temp dir");
    let c1_dir = dir.mkdir().expect("could not create child dir");
    let c2_dir = dir.mkdir().expect("could not create child dir");

    let rid = container::init(dir.path()).expect("could not init root `Container`");
    let cid_1 = container::init(&c1_dir).expect("could not init child `Container`");
    let cid_2 = container::init(&c2_dir).expect("could not init child `Container`");

    // test
    let graph = ContainerTreeLoader::load(dir.path()).expect("could not load `Container` tree");

    assert_eq!(&rid, graph.root(), "incorrect root");
    assert!(
        graph
            .children(graph.root())
            .expect("root children not found")
            .contains(&cid_1),
        "child `Container` not a child"
    );

    assert!(
        graph
            .children(graph.root())
            .expect("root children not found")
            .contains(&cid_2),
        "child `Container` not a child"
    );

    assert!(graph.get(&rid).is_some(), "root `Container` not loaded");
    assert!(graph.get(&cid_1).is_some(), "child `Container` not loaded");
    assert!(graph.get(&cid_2).is_some(), "child `Container` not loaded");
}

#[test]
fn container_tree_duplicate_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create temp dir");
    let c1_dir = dir.mkdir().expect("could not create child dir");
    let c2_dir = dir.mkdir().expect("could not create child dir");

    let c1_tdir = dir
        .children
        .get_mut(&c1_dir)
        .expect("could not get child dhirectory");

    let c11_dir = c1_tdir.mkdir().expect("could not create child dir");
    let c12_dir = c1_tdir.mkdir().expect("could not create child dir");

    let _rid = container::init(dir.path()).expect("could not init root `Container`");
    let cid_1 = container::init(&c1_dir).expect("could not init child `Container`");
    let _cid_2 = container::init(&c2_dir).expect("could not init child `Container`");
    let _cid_11 = container::init(&c11_dir).expect("could not init grandchild `Container`");
    let _cid_12 = container::init(&c12_dir).expect("could not init grandchild `Container`");

    let graph = ContainerTreeLoader::load(dir.path()).expect("could not load `Container` tree");

    // test
    let dup = graph
        .duplicate(graph.root())
        .expect("could not duplicate tree from root");

    let root_children = dup
        .children(graph.root())
        .expect("could not get root children");

    assert_eq!(2, root_children.len(), "incorrect number of children");

    let c_dup = graph
        .duplicate(&cid_1)
        .expect("could not duplicate tree from root");

    let c1_children = c_dup
        .children(graph.root())
        .expect("could not get root children");

    assert_eq!(2, c1_children.len(), "incorrect number of children");
}

#[test]
fn container_tree_set_base_path_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create temp dir");
    let c1_dir = dir.mkdir().expect("could not create child dir");
    let c2_dir = dir.mkdir().expect("could not create child dir");

    let c1_tdir = dir
        .children
        .get_mut(&c1_dir)
        .expect("could not get child dhirectory");

    let c11_dir = c1_tdir.mkdir().expect("could not create child dir");
    let c12_dir = c1_tdir.mkdir().expect("could not create child dir");
    let c1_dir_new = dir.mkdir().expect("could not create child dir");
    let mut c11_dir_new = c1_dir_new.clone();
    let mut c12_dir_new = c1_dir_new.clone();
    c11_dir_new.push(c11_dir.file_name().expect("could not get file name"));
    c12_dir_new.push(c12_dir.file_name().expect("could not get file name"));

    let _rid = container::init(dir.path()).expect("could not init root `Container`");
    let cid_1 = container::init(&c1_dir).expect("could not init child `Container`");
    let cid_2 = container::init(&c2_dir).expect("could not init child `Container`");
    let cid_11 = container::init(&c11_dir).expect("could not init grandchild `Container`");
    let cid_12 = container::init(&c12_dir).expect("could not init grandchild `Container`");

    let mut graph = ContainerTreeLoader::load(dir.path()).expect("could not load `Container` tree");

    // test
    graph
        .set_base_path(&cid_1, c1_dir_new.clone())
        .expect("could not set new base path");

    assert_eq!(
        dir.path(),
        &graph
            .get(graph.root())
            .expect("could not get graph root")
            .base_path()
            .expect("base path not set"),
        "root path should not change"
    );

    assert_eq!(
        c1_dir,
        graph
            .get(&cid_1)
            .expect("could not get `Container`")
            .base_path()
            .expect("base path not set"),
        "child dir path should be changed"
    );

    assert_eq!(
        c2_dir,
        graph
            .get(&cid_2)
            .expect("could not get `Container`")
            .base_path()
            .expect("base path not set"),
        "child dir path should not be changed"
    );

    assert_eq!(
        c11_dir_new,
        graph
            .get(&cid_11)
            .expect("could not get `Container`")
            .base_path()
            .expect("base path not set"),
        "grandchild dir path should be changed"
    );

    assert_eq!(
        c12_dir,
        graph
            .get(&cid_12)
            .expect("could not get `Container`")
            .base_path()
            .expect("base path not set"),
        "grandchild dir path should be changed"
    );
}
