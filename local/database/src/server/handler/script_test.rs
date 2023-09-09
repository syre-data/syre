use super::*;
use dev_utils::fs::TempDir;
use dev_utils::path::resource_path;
use thot_core::project::{Script, ScriptAssociation};
use thot_local::project::resources::{Project as LocalProject, Scripts as LocalScripts};
use thot_local::project::{container, project};

#[test]
fn remove_script_should_work() {
    // setup

    let mut _dir = TempDir::new().expect("could not create new `TempDir`");
    let child_dir = _dir.mkdir().expect("could not create new child directory");

    // initialize project
    project::init(_dir.path()).expect("could not init `Project`");
    let mut project = LocalProject::load_from(_dir.path()).expect("could not load `Project`");

    let cid = container::init(_dir.path()).expect("could not init `Container`");

    let child_cid = container::init(&child_dir).expect("could not init child `Container`");

    let mut container = LocalContainer::load_from(_dir.path()).expect("could not load `Container`");

    let mut child_container =
        LocalContainer::load_from(&child_dir).expect("could not load child `Container`");

    container.register_child(child_cid.clone());

    project.data_root = Some(_dir.path().to_path_buf());
    let pid = project.rid.clone();

    // initialize scripts

    let script_0 =
        Script::new(resource_path::resource_path(Some("py"))).expect("could not create `script`");

    let script_1 =
        Script::new(resource_path::resource_path(Some("py"))).expect("could not create `script` 1");

    let sid_0 = script_0.rid.clone();
    let sid_1 = script_1.rid.clone();

    let mut scripts = LocalScripts::load_from(_dir.path()).expect("could not load `Scripts`");

    scripts
        .insert_script(script_0)
        .expect("could not insert `Script`");

    scripts
        .insert_script(script_1)
        .expect("could not insert `Script` 1");

    // add script association

    let assoc_root_0 = ScriptAssociation::new(sid_0.clone());
    let assoc_root_1 = ScriptAssociation::new(sid_1.clone());
    let assoc_child_0 = ScriptAssociation::new(sid_0.clone());
    let assoc_child_1 = ScriptAssociation::new(sid_1.clone());

    container
        .add_script_association(assoc_root_0.clone())
        .expect("could not add `ScriptAssociation`");

    container
        .add_script_association(assoc_root_1.clone())
        .expect("could not add `ScriptAssociation`");

    child_container
        .add_script_association(assoc_child_0.clone())
        .expect("could not add `ScriptAssociation`");

    child_container
        .add_script_association(assoc_child_1.clone())
        .expect("could not add `ScriptAssociation`");

    project.save().expect("could not save `Project`");
    scripts.save().expect("could not save `Scripts`");

    container.save().expect("could not save root `Container`");
    child_container
        .save()
        .expect("could not save child `Container`");

    // database setup

    drop(child_container);
    container.load_children(true).expect("could not load child");

    let mut db = Database::new();
    db.store
        .insert_project(project)
        .expect("could not insert `Project`");

    db.store
        .insert_container_tree(Arc::new(Mutex::new(container)))
        .expect("could not insert `Container`");

    db.store.insert_project_scripts(pid.clone(), scripts);

    // test

    db.remove_script(&pid, &sid_0)
        .expect("could not remove `Script`");

    let scripts = db
        .store
        .get_project_scripts(&pid)
        .expect("could not get `Project`");

    // scripts are properly removed from project
    assert!(!scripts.contains_key(&sid_0));

    assert!(scripts.contains_key(&sid_1));

    let container = db
        .store
        .get_container(&cid)
        .expect("could not get `Container`");

    let child_container = db
        .store
        .get_container(&child_cid)
        .expect("could not get `Container`");

    let container = container.lock().expect("could not lock `Container`");
    let child_container = child_container.lock().expect("could not lock `Container`");

    // scripts are properly removed from container
    assert!(
        !container.scripts.contains_key(&sid_0),
        "root container should not contain removed script"
    );
    assert!(
        container.scripts.contains_key(&sid_1),
        "root container should contain unremoved script"
    );
    assert!(
        !child_container.scripts.contains_key(&sid_0),
        "child container should not contain removed script"
    );
    assert!(
        child_container.scripts.contains_key(&sid_1),
        "child container should contain unremoved script"
    );
    // TODO ensure changes save to disk
}
