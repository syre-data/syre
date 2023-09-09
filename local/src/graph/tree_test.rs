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
fn container_tree_duplicate_to_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create temp dir");
    let c1_dir = dir.mkdir().expect("could not create child dir");
    let c2_dir = dir.mkdir().expect("could not create child dir");
    let dup_dir = TempDir::new().unwrap();
    let dup_child_dir = TempDir::new().unwrap();

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
    let dup = ContainerTreeDuplicator::duplicate_to(dup_dir.path(), &graph, graph.root())
        .expect("could not duplicate tree from root");

    let root_children = dup
        .children(graph.root())
        .expect("could not get root children");

    assert_eq!(2, root_children.len(), "incorrect number of children");

    let c_dup = ContainerTreeDuplicator::duplicate_to(dup_child_dir.path(), &graph, &cid_1)
        .expect("could not duplicate tree from root");

    let c1_children = c_dup
        .children(graph.root())
        .expect("could not get root children");

    assert_eq!(2, c1_children.len(), "incorrect number of children");
}
