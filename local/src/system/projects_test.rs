// @todo: Tests must be run with `--test-threads=1`.
use super::*;
use crate::Error;
use dev_utils::{create_lock, lock::get_lock};
use fake::faker::filesystem::raw::DirPath;
use fake::locales::EN;
use fake::Fake;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_core::types::ResourceId;

#[test]
fn register_project_should_work() {
    let _m = get_lock(&MTX);

    let (prj, path) = create_project();
    let rid = prj.clone();
    register_project(prj, path).expect("register__project should work");

    let mut projects = Projects::load().expect("could not load Projects");
    let project = projects.get(&rid);
    assert!(project.is_some(), "project not registered");

    // clean up
    projects.remove(&rid);
    projects
        .save()
        .expect("could not save `Projects` during clean up");
}

#[test]
fn register_project_should_error_if_id_already_exists() {
    let _m = get_lock(&MTX);

    let (prj0, path) = create_project();
    let prj1 = prj0.clone();

    register_project(prj0, path.clone()).expect("register_project should work");
    match register_project(prj1, path) {
        Err(Error::CoreError(CoreError::ResourceError(ResourceError::DuplicateId(_)))) => {} // expected error
        res => panic!(
            "Unexpected result. Expected duplicate id error found {:?}",
            res
        ),
    };
}

#[test]
fn deregister_project_should_work() {
    let _m = get_lock(&MTX);

    let (prj, path) = create_project();
    register_project(prj.clone(), path).expect("register_project should work");
    deregister_project(&prj).expect("deregister_project should work");

    let projects = Projects::load().expect("could not load projects");
    let project = projects.get(&prj);

    assert!(project.is_none(), "project not removed");
}

#[test]
fn deregister_project_should_exit_silently_if_project_did_not_exist() {
    let _m = get_lock(&MTX);

    let (prj, _path) = create_project();
    deregister_project(&prj).expect("deregister_project should work");
}

// ************************
// *** helper functions ***
// ************************

fn create_project() -> (ResourceId, PathBuf) {
    let rid = ResourceId::new();
    let path = DirPath(EN).fake();
    (rid, path)
}

create_lock!(MTX);
