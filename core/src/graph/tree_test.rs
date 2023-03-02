use super::*;
use crate::types::ResourceId;
use has_id::HasId;
use rand::Rng;

// *************
// *** tests ***
// *************

#[test]
fn new_should_work() {
    let root = Data::new();
    ResourceTree::new(root);
}

#[test]
fn insert_should_work() {
    // setup
    let root = Data::new();
    let mut tree = ResourceTree::new(root);

    let c1 = Data::new();
    let c2 = Data::new();
    let c11 = Data::new();

    let root = tree.root().clone();
    let c1_id = c1.id().clone();
    let c2_id = c2.id().clone();
    let c11_id = c11.id().clone();

    // test
    tree.insert(root.clone(), c1)
        .expect("could not insert root child `Node`");

    assert!(
        tree.children(&root)
            .expect("root `Node` not found")
            .contains(&c1_id),
        "root `Node` should have edge to child `Node`"
    );

    assert_eq!(
        Some(&root),
        tree.parent(&c1_id).expect("parent `Node` not found"),
        "child `Node` parent incorrect"
    );

    tree.insert(root.clone(), c2)
        .expect("could not insert root child `Node`");

    assert!(
        tree.children(&root)
            .expect("root `Node` not found")
            .contains(&c2_id),
        "root `Node` should have edge to child `Node`"
    );

    assert_eq!(
        Some(&root),
        tree.parent(&c2_id).expect("parent `Node` not found"),
        "child `Node` parent incorrect"
    );

    tree.insert(c1_id.clone(), c11)
        .expect("could not insert grandchild `Node`");

    assert!(
        tree.children(&c1_id)
            .expect("child `Node` not found")
            .contains(&c11_id),
        "child `Node` should have edge to grandchild `Node`"
    );

    assert_eq!(
        Some(&c1_id),
        tree.parent(&c11_id).expect("parent `Node` not found"),
        "child `Node` parent incorrect"
    );

    assert!(
        !tree
            .children(&root)
            .expect("root `Node` not found")
            .contains(&c11_id),
        "root `Node` should not have edge to grandchild `Node`"
    );
}

#[test]
fn get_should_work() {
    // setup
    let root = Data::new();
    let data_0 = root.inner().clone();
    let tree = ResourceTree::new(root);
    let root = tree.root();

    // test
    let f_root = tree.get(root).expect("root `Node` should exist");
    assert_eq!(&data_0, f_root.inner(), "root data incorrect");
}

#[test]
fn root_should_work() {
    // setup
    let root = Data::new();
    let data_0 = root.inner().clone();
    let tree = ResourceTree::new(root);

    // test
    let root_id = tree.root();
    let f_root = tree.get(root_id).expect("root `Node` should exist");
    assert_eq!(&data_0, f_root.inner(), "root data incorrect");
}

#[test]
fn insert_tree_should_work() {
    // setup
    let mut tree = create_tree();
    let sub_tree = create_tree();

    let root = tree.root().clone();
    let sub_root = sub_tree.root().clone();

    let c1_id = tree
        .children(&root)
        .expect("could not get children of root `Node`")
        .get_index(0)
        .expect("root `Node` does not have child")
        .clone();

    let sub_root_children = sub_tree
        .children(&sub_root)
        .expect("could not get children of root `Node`")
        .clone();

    // test
    tree.insert_tree(&c1_id, sub_tree)
        .expect("could not insert tree");

    assert!(
        tree.children(&c1_id)
            .expect("could not get `Node` children")
            .contains(&sub_root),
        "subtree not added"
    );

    let root_parent = tree.parent(tree.root()).expect("parent `Node` not found");

    assert!(root_parent.is_none());
    assert_eq!(
        Some(&c1_id),
        tree.parent(&sub_root).expect("`Node` parent not found"),
        "subtree root parent not correct"
    );

    let sub_children = tree
        .children(&sub_root)
        .expect("could not get `Node` children");

    for child in sub_root_children {
        assert!(sub_children.contains(&child), "subtree child not inserted");
        assert_eq!(
            Some(&sub_root),
            tree.parent(&child).expect("`Node` parent not found"),
            "parent edge not found"
        );
    }
}

#[test]
fn remove_should_work() {
    // setup
    let mut tree = create_tree();
    let c1 = tree
        .children(tree.root())
        .expect("could not get root `Node` children")
        .get_index(0)
        .expect("`Node` has no children")
        .clone();

    let c1_children = tree
        .children(&c1)
        .expect("could not get `Node` children")
        .clone();

    // test
    let sub_tree = tree.remove(&c1).expect("could not remove tree");
    assert_eq!(&c1, sub_tree.root(), "incorrect tree removed");
    assert!(
        tree.parent(&c1).is_err(),
        "removed `Node` parent should error"
    );

    let sub_children = sub_tree
        .children(sub_tree.root())
        .expect("could not get subtree root `Node` children");

    assert_eq!(&c1_children, sub_children, "children do not match");

    for child in sub_children {
        assert_eq!(
            Some(&c1),
            sub_tree
                .parent(&child)
                .expect("could not get parent of `Node`"),
            "incorrect parent"
        );

        assert!(
            tree.parent(&child).is_err(),
            "removed `Node` should not have parent"
        );
    }
}

#[test]
fn mv_should_work() {
    // setup
    let root = Data::new();
    let mut tree = ResourceTree::new(root);

    let c1 = Data::new();
    let c2 = Data::new();
    let c11 = Data::new();
    let c111 = Data::new();

    let c1_id = c1.id().clone();
    let c2_id = c2.id().clone();
    let c11_id = c11.id().clone();
    let c111_id = c111.id().clone();

    tree.insert(tree.root().clone(), c1)
        .expect("could not insert root child `Node`");

    tree.insert(tree.root().clone(), c2)
        .expect("could not insert root child `Node`");

    tree.insert(c1_id.clone(), c11)
        .expect("could not insert `Node`");

    tree.insert(c11_id.clone(), c111)
        .expect("could not insert `Node`");

    // test
    tree.mv(&c11_id, &c2_id).expect("could not move tree");

    assert!(
        tree.children(&c2_id)
            .expect("could not get `Node` children")
            .contains(&c11_id),
        "`Node` not moved"
    );

    assert_eq!(
        Some(&c2_id),
        tree.parent(&c11_id).expect("`Node` parent not found"),
        "`Node` parent not set"
    );

    assert!(tree
        .children(&c11_id)
        .expect("could not get `Node` children")
        .contains(&c111_id));

    assert_eq!(
        Some(&c11_id),
        tree.parent(&c111_id).expect("`Node` parent not found"),
        "`Node` parent not set"
    );
}

#[test]
fn move_index_should_work() {
    // setup
    let mut tree = create_tree();
    let children = tree
        .children(tree.root())
        .expect("could not get root `Node` children");

    let c1 = children
        .get_index(0)
        .expect("could not get child `Node` by index")
        .clone();

    drop(children);

    // test
    tree.move_index(&c1, 1)
        .expect("could not move child `Node` index");

    let children = tree
        .children(tree.root())
        .expect("could not get root `Node` children");

    assert_eq!(
        &c1,
        children.get_index(1).expect("child `Node` not found"),
        "`Node` in incorrect position"
    );
}

// *****************
// *** Mock Data ***
// *****************

#[derive(HasId)]
struct Data {
    #[id]
    id: ResourceId,
    inner: i32,
}

impl Data {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            id: ResourceId::new(),
            inner: rng.gen(),
        }
    }

    pub fn inner(&self) -> &i32 {
        &self.inner
    }
}

// ***************
// *** helpers ***
// ***************

fn create_tree() -> ResourceTree<Data> {
    let root = Data::new();
    let mut tree = ResourceTree::new(root);

    let c1 = Data::new();
    let c2 = Data::new();
    let c11 = Data::new();
    let c1_id = c1.id().clone();

    tree.insert(tree.root().clone(), c1)
        .expect("could not insert root child `Node`");

    tree.insert(tree.root().clone(), c2)
        .expect("could not insert root child `Node`");

    tree.insert(c1_id.clone(), c11)
        .expect("could not insert root child `Node`");

    tree
}
