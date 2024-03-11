use super::*;
use crate::project::{container, project};
use crate::system::scripts;
use dev_utils::fs::TempDir;

#[test]
fn add_association_should_work() {
    // setup
    let mut _dir = init_project().unwrap();
    let builder = container::InitOptions::init();
    let _cid = builder.build(_dir.path()).unwrap();
    let s_path = _dir.mkfile_with_extension("py").unwrap();
    let sid = scripts::make_script(&s_path).unwrap();

    // test
    let _rid = add_association(&sid, _dir.path()).unwrap();
    let container = ContainerLoader::load(_dir.path()).unwrap();
    assert_eq!(
        1,
        container.analyses.len(),
        "container should has script association added"
    );
}

#[test]
#[should_panic(expected = "PathNotAContainer")]
fn add_association_outside_container_should_error() {
    // setup
    let mut _dir = init_project().unwrap();
    let s_path = _dir.mkfile_with_extension("py").unwrap();
    let sid = scripts::make_script(&s_path).unwrap();

    // test
    add_association(&sid, _dir.path()).unwrap();
}

// ************************
// *** helper functions ***
// ************************

/// Initialize a project in a new temporary directory.
fn init_project() -> Result<TempDir> {
    let _dir = TempDir::new().unwrap();
    project::init(_dir.path()).unwrap();

    Ok(_dir)
}
