use super::*;
use dev_utils::fs::TempDir;
use dev_utils::path::resource_path;
use thot_core::project::{Script, ScriptAssociation};
use thot_local::project::resources::{Project as LocalProject, Scripts as LocalScripts};
use thot_local::project::{container, project};

#[test]
fn remove_script_should_work() {
    // setup

    let _dir = TempDir::new().expect("could not create new `TempDir`");

    // initialize project
    project::init(_dir.path()).expect("could not init `Project`");
    let mut project = LocalProject::load(_dir.path()).expect("could not load `Project`");

    let cid = container::init(_dir.path()).expect("could not init `Container`");
    let mut container = LocalContainer::load(_dir.path()).expect("could not load `Container`");

    project.data_root = Some(_dir.path().to_path_buf());
    let pid = project.rid.clone();

    // initialize scripts

    let mut scripts = LocalScripts::load(_dir.path()).expect("could not load `Scripts`");

    let script =
        Script::new(resource_path::resource_path(Some("py"))).expect("could not create `script`");

    let sid = script.rid.clone();

    scripts
        .insert_script(script)
        .expect("could not insert `Script`");

    // add script association

    let script_association = ScriptAssociation::new(sid.clone());
    container
        .add_script_association(script_association)
        .expect("could not add `ScriptAssociation`");

    project.save().expect("could not save `Project`");
    scripts.save().expect("could not save `Scripts`");

    // database setup

    let mut db = Database::new();
    db.store
        .insert_project(project)
        .expect("could not insert `Project`");

    db.store
        .insert_container_tree(Arc::new(Mutex::new(container)))
        .expect("could not insert `Container`");

    db.store.insert_project_scripts(pid.clone(), scripts);

    // test

    db.remove_script(&pid, &sid)
        .expect("could not remove `Script`");

    let scripts = db
        .store
        .get_project_scripts(&pid)
        .expect("could not get `Project`");

    // scripts are removed from project
    assert!(!scripts.contains_key(&sid));

    let container = db
        .store
        .get_container(&cid)
        .expect("could not get `Container`");

    let container = container.lock().expect("could not lock `Container`");

    // scripts are removed from container
    assert!(!container.scripts.contains_key(&sid));
    // @todo[3]: ensure changes save to disk
}
