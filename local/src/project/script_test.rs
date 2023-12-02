use super::*;
use crate::project::resources::Container;
use crate::project::{container, project};
use crate::system::scripts;
use dev_utils::fs::TempDir;

#[test]
fn add_association_should_work() {
    // setup
    let mut _dir = init_project().expect("setup should work");
    let builder = container::InitOptions::init();
    let _cid = builder
        .build(_dir.path())
        .expect("init as container should work");
    let s_path = _dir
        .mkfile_with_extension("py")
        .expect("mkfile should work");

    let sid = scripts::make_script(&s_path).expect("could not register script");

    // test
    let _rid = add_association(&sid, _dir.path()).expect("add association should work");
    let container = Container::load_from(_dir.path()).expect("load container should work");
    assert_eq!(
        1,
        container.scripts.len(),
        "container should has script association added"
    );
}

#[test]
#[should_panic(expected = "PathNotAContainer")]
fn add_association_outside_container_should_error() {
    // setup
    let mut _dir = init_project().expect("setup should work");
    let s_path = _dir
        .mkfile_with_extension("py")
        .expect("could not create file");

    let sid = scripts::make_script(&s_path).expect("could not register script");

    // test
    add_association(&sid, _dir.path()).unwrap();
}

// ************************
// *** helper functions ***
// ************************

/// Initialize a project in a new temporary directory.
fn init_project() -> Result<TempDir> {
    let _dir = TempDir::new().expect("temp dir should work");
    project::init(_dir.path()).expect("init project should work");

    Ok(_dir)
}
