use std::fs;
use std::thread;
use syre_local::project::resources::Project as LocalProject;
use syre_local_database::server::Database;
use syre_local_database::Client;

#[test]
fn test_server_commands() {
    // setup
    thread::spawn(|| {
        let mut db = Database::new().unwrap();
        db.start();
    });

    // wait for server to start
    let mut i = 0;
    while !Client::server_available() {
        if i > 9 {
            panic!("server not available");
        }

        i += 1;
        thread::sleep(std::time::Duration::from_millis(100));
    }

    let project_path = common::init_project();
    let project_path = fs::canonicalize(project_path).unwrap();
    let project = LocalProject::load_from(&project_path).unwrap();
    common::init_project_graph(project);

    let db = Client::new();

    // test
    let (project, _settings) = db
        .project()
        .load_with_settings(project_path.clone())
        .unwrap()
        .expect("could not load project");

    db.project().get(project.rid.clone()).unwrap().unwrap();
    let path = db.project().path(project.rid.clone()).unwrap();
    assert_eq!(path.as_ref(), Some(&project_path));

    let graph = db
        .graph()
        .get_or_load(project.rid.clone())
        .unwrap()
        .expect("could not get graph");

    let children = db
        .graph()
        .children(graph.root().clone())
        .unwrap()
        .expect("could not get children");
    assert_eq!(children.len(), 0);

    let parent = db
        .graph()
        .parent(graph.root().clone())
        .unwrap()
        .expect("could not get parent");
    assert!(parent.is_none());

    let root = db
        .container()
        .get(graph.root().clone())
        .unwrap()
        .expect("container not found");
    assert_eq!(&root.rid, graph.root());

    let path = db
        .container()
        .path(root.rid.clone())
        .unwrap()
        .expect("container not found");
    assert_eq!(path, project_path.join(project.data_root));

    // cleanup
    fs::remove_dir_all(project_path).unwrap();
}

mod common {
    use std::fs;
    use std::path::PathBuf;
    use syre_local::project::project;
    use syre_local::project::resources::{Container as LocalContainer, Project as LocalProject};

    pub fn init_project() -> PathBuf {
        let project_dir = tempfile::tempdir().unwrap();
        project::init(project_dir.path()).unwrap();
        project_dir.into_path()
    }

    pub fn init_project_graph(prj: LocalProject) {
        fs::create_dir(prj.data_root_path()).unwrap();
        let root = LocalContainer::new(prj.data_root_path());
        root.save().unwrap();
    }
}
