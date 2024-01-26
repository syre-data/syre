use super::*;
use crate::project::container;
use dev_utils::fs::TempDir;

#[test]
fn container_tree_load_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create temp dir");
    let c1_dir = dir.mkdir().expect("could not create child dir");
    let c2_dir = dir.mkdir().expect("could not create child dir");

    let builder = container::InitOptions::init();
    let rid = builder
        .build(dir.path())
        .expect("could not init root `Container`");

    let cid_1 = builder
        .build(&c1_dir)
        .expect("could not init child `Container`");

    let cid_2 = builder
        .build(&c2_dir)
        .expect("could not init child `Container`");

    // test
    let graph = Loader::load(dir.path()).expect("could not load `Container` tree");

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
