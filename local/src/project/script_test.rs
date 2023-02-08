use super::*;
use crate::project::resources::Container;
use crate::project::{container, project};
use crate::system::scripts;
use dev_utils::fs::TempDir;

// **************
// *** Script ***
// **************

// #[test]
// fn init_with_default_project_should_work() {
//     // setup
//     let mut _dir = init_project().expect("setup should work");
//     let s_path = _dir
//         .mkfile_with_extension("py")
//         .expect("mkfile should work");

//     // test
//     let rid = init(s_path.as_path(), None).expect("init should work");
//     let scripts = Scripts::load(_dir.path()).expect("load scripts should work");
//     assert!(scripts.contains(&rid), "script should be registered");
// }

// #[test]
// fn init_with_specified_project_should_work() {
//     // setup
//     let mut _dir = TempDir::new().expect("setup should work");
//     let p_dir = _dir
//         .mkdir()
//         .expect("creating project directory should work");
//     let s_dir = _dir.mkdir().expect("creating script directory should work");

//     project::init(&p_dir).expect("init project should work");
//     let mut s_path = s_dir.join(FileName(EN).fake::<String>());
//     s_path.set_extension(".py");

//     // test
//     let rid = init(&s_path, Some(&p_dir)).expect("init should work");
//     let scripts = Scripts::load(&p_dir).expect("load scripts should work");
//     assert!(scripts.contains(&rid), "script should be registered");
// }

// #[test]
// fn init_should_do_nothing_if_path_already_initialized_as_script() {
//     // setup
//     let mut _dir = init_project().expect("setup should work");
//     let s_path = _dir
//         .mkfile_with_extension("py")
//         .expect("mkfile should work");
//     let rid = init(&s_path, None).expect("initial script init should work");

//     // test
//     let found = init(&s_path, None).expect("second script init should work");
//     assert_eq!(rid, found, "resource ids should match");
// }

// #[test]
// #[should_panic(expected = "PathNotInProject")]
// fn init_with_default_project_but_not_in_project_should_error() {
//     // setup
//     let _dir = TempDir::new().expect("setup should work");
//     let s_path = _dir.path().join(FileName(EN).fake::<String>());

//     // test
//     init(&s_path, None).unwrap();
// }

// #[test]
// #[should_panic(expected = "PathNotAProjectRoot")]
// fn init_if_project_root_is_invalid_should_error() {
//     // setup
//     let _dir = TempDir::new().expect("setup should work");
//     let s_path = create_path_in(_dir.path());

//     // test
//     init(&s_path.as_path(), Some(_dir.path())).unwrap();
// }

// **************************
// *** Script Association ***
// **************************

#[test]
fn add_association_should_work() {
    // setup
    let mut _dir = init_project().expect("setup should work");
    let _cid = container::init(_dir.path()).expect("init as container should work");
    let s_path = _dir
        .mkfile_with_extension("py")
        .expect("mkfile should work");

    let sid = scripts::make_script(&s_path).expect("could not register script");

    // test
    let _rid = add_association(&sid, _dir.path()).expect("add association should work");
    let container = Container::load(_dir.path()).expect("load container should work");
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
