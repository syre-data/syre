use super::*;
use crate::server::Builder;
use syre_core::graph::ResourceTree;
use syre_core::project::{AnalysisAssociation, Script};
use syre_local::project::resources::{
    Analyses as LocalScripts, Container as LocalContainer, Project as LocalProject,
};

#[test]
fn remove_script_should_work() {
    // setup
    let dir = tempfile::tempdir().unwrap();
    let data_dir = tempfile::tempdir_in(dir.path()).unwrap();
    let child_dir = tempfile::tempdir_in(data_dir.path()).unwrap();

    // initialize project
    let mut project = LocalProject::new(dir.path()).unwrap();
    let mut container = LocalContainer::new(data_dir.path());
    let mut child_container = LocalContainer::new(child_dir.path());

    project.data_root = data_dir.path().to_path_buf();
    let cid = container.rid.clone();
    let child_cid = child_container.rid.clone();
    let pid = project.rid.clone();

    // initialize scripts
    let script_0 = Script::from_path("script-0.py").unwrap();
    let script_1 = Script::from_path("script-1.py").unwrap();
    let sid_0 = script_0.rid.clone();
    let sid_1 = script_1.rid.clone();

    let mut scripts = LocalScripts::load_from(dir.path()).expect("could not load `Scripts`");
    scripts
        .insert_script_unique_path(script_0)
        .expect("could not insert `Script`");

    scripts
        .insert_script_unique_path(script_1)
        .expect("could not insert `Script` 1");

    // add script association
    let assoc_root_0 = AnalysisAssociation::new(sid_0.clone());
    let assoc_root_1 = AnalysisAssociation::new(sid_1.clone());
    let assoc_child_0 = AnalysisAssociation::new(sid_0.clone());
    let assoc_child_1 = AnalysisAssociation::new(sid_1.clone());

    container
        .add_analysis_association(assoc_root_0.clone())
        .expect("could not add `ScriptAssociation`");

    container
        .add_analysis_association(assoc_root_1.clone())
        .expect("could not add `ScriptAssociation`");

    child_container
        .add_analysis_association(assoc_child_0.clone())
        .expect("could not add `ScriptAssociation`");

    child_container
        .add_analysis_association(assoc_child_1.clone())
        .expect("could not add `ScriptAssociation`");

    let mut graph = ResourceTree::new(container);
    graph.insert(cid.clone(), child_container).unwrap();

    // database setup
    let db = Builder::default();
    db.object_store.insert_project(project).unwrap();
    db.object_store
        .insert_project_graph_canonical(pid.clone(), graph)
        .unwrap();

    db.object_store.insert_project_scripts(pid.clone(), scripts);

    // test

    db.remove_analysis(&pid, &sid_0).unwrap();
    let scripts = db.object_store.get_project_scripts(&pid).unwrap();

    // scripts are properly removed from project
    assert!(!scripts.contains_key(&sid_0));
    assert!(scripts.contains_key(&sid_1));

    let container = db.object_store.get_container(&cid).unwrap();
    let child_container = db.object_store.get_container(&child_cid).unwrap();

    // scripts are properly removed from container
    assert!(
        !container.analyses.contains_key(&sid_0),
        "root container should not contain removed script"
    );
    assert!(
        container.analyses.contains_key(&sid_1),
        "root container should contain unremoved script"
    );
    assert!(
        !child_container.analyses.contains_key(&sid_0),
        "child container should not contain removed script"
    );
    assert!(
        child_container.analyses.contains_key(&sid_1),
        "child container should contain unremoved script"
    );
    // TODO ensure changes save to disk
}
