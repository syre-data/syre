use super::*;
use dev_utils::fs::TempDir;
use dev_utils::path::resource_path;
use thot_core::graph::ResourceTree;
use thot_core::project::{Script, ScriptAssociation};
use thot_local::project::resources::{
    Container as LocalContainer, Project as LocalProject, Scripts as LocalScripts,
};

#[test]
fn remove_script_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create new `TempDir`");
    let data_dir = dir.mkdir().expect("could not create new child directory");
    let child_dir = dir.children.get_mut(&data_dir).unwrap().mkdir().unwrap();

    // initialize project
    let mut project = LocalProject::new(dir.path()).unwrap();
    let mut container = LocalContainer::new(data_dir.clone());
    let mut child_container = LocalContainer::new(child_dir.clone());

    project.data_root = Some(data_dir.clone());
    let cid = container.rid.clone();
    let child_cid = child_container.rid.clone();
    let pid = project.rid.clone();

    // initialize scripts
    let script_0 =
        Script::new(resource_path::resource_path(Some("py"))).expect("could not create `script`");

    let script_1 =
        Script::new(resource_path::resource_path(Some("py"))).expect("could not create `script` 1");

    let sid_0 = script_0.rid.clone();
    let sid_1 = script_1.rid.clone();

    let mut scripts = LocalScripts::load_from(dir.path()).expect("could not load `Scripts`");

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

    let mut graph = ResourceTree::new(container);
    graph.insert(cid.clone(), child_container).unwrap();

    // database setup
    let mut db = Database::new();
    db.store
        .insert_project(project)
        .expect("could not insert `Project`");

    db.store
        .insert_project_graph_canonical(pid.clone(), graph)
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
