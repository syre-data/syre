use super::*;
use crate::command::{ContainerCommand, GraphCommand, ProjectCommand};
use crate::error::Result;
use dev_utils::fs::TempDir;
use fake::faker::lorem::raw::Word;
use fake::locales::EN;
use fake::Fake;
use std::path::Path;
use thot_core::project::{Container as CoreContainer, ContainerProperties};
use thot_local::loader::container::Loader as ContainerLoader;
use thot_local::project::resources::Project as LocalProject;
use thot_local::project::{container, project};

#[test]
fn database_command_update_container_properties_without_name_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create new temp dir");
    let data_dir = dir.mkdir().unwrap();
    let pid = project::init(dir.path()).unwrap();
    let builder = container::InitOptions::new();
    let _rid = builder
        .build(&data_dir)
        .expect("could not init `Container`");

    let mut db = Database::new();
    let _project = db.handle_command_project(ProjectCommand::Load(dir.path().into()));
    let container = db.handle_command_graph(GraphCommand::Load(pid));

    let container: Result<CoreContainer> =
        serde_json::from_value(container).expect("could not contvert JsValue to `Container`");

    let container = container.expect("`LoadContainer` should work");
    let mut properties = container.properties.clone();
    let kind = Word(EN).fake();
    properties.kind = kind;

    // test
    db.handle_command_container(ContainerCommand::UpdateProperties(UpdatePropertiesArgs {
        rid: container.rid.clone(),
        properties: properties.clone(),
    }));

    {
        // ensure stored container updated
        let stored = db
            .store
            .get_container(&container.rid)
            .expect("`Container` not stored");

        assert_eq!(
            properties.name, stored.properties.name,
            "incorrect name stored"
        );
    }

    let saved = ContainerLoader::load(dir.path()).expect("could not load `Container`");
    assert_eq!(
        properties.name, saved.properties.name,
        "incorrect name persisted"
    );
}

#[test]
fn database_command_update_container_name_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create new temp dir");
    let data_dir = dir.mkdir().unwrap();
    let child_dir = dir.children.get_mut(&data_dir).unwrap().mkdir().unwrap();
    let sibling_dir = dir.children.get_mut(&data_dir).unwrap().mkdir().unwrap();

    let mut project = LocalProject::new(dir.path().to_path_buf()).unwrap();
    project.data_root = Some(data_dir.clone());
    project.save().unwrap();

    let pid = project.rid.clone();
    let builder = container::InitOptions::new();
    let rid = builder
        .build(&data_dir)
        .expect("could not init `Container`");
    let cid = builder
        .build(&child_dir)
        .expect("could not init `Container`");
    let _sid = builder
        .build(&sibling_dir)
        .expect("could not init `Container`");

    let mut db = Database::new();
    let _project = db.handle_command_project(ProjectCommand::Load(dir.path().into()));
    let _graph = db.handle_command_graph(GraphCommand::Load(pid));

    let properties = ContainerProperties::new(Word(EN).fake::<String>());

    // test
    // root
    db.handle_command_container(ContainerCommand::UpdateProperties(UpdatePropertiesArgs {
        rid: rid.clone(),
        properties: properties.clone(),
    }));

    let stored = db
        .store
        .get_container(&rid)
        .expect("`Container` not stored");

    assert_eq!(
        properties.name, stored.properties.name,
        "incorrect name stored"
    );

    assert_eq!(&data_dir, stored.base_path(), "root path should not change");

    let saved = ContainerLoader::load(&data_dir).expect("could not load `Container`");
    assert_eq!(
        properties.name, saved.properties.name,
        "incorrect name persisted"
    );

    // child
    db.handle_command_container(ContainerCommand::UpdateProperties(UpdatePropertiesArgs {
        rid: cid.clone(),
        properties: properties.clone(),
    }));

    let stored = db
        .store
        .get_container(&cid)
        .expect("`Container` not stored");

    assert_eq!(
        properties.name, stored.properties.name,
        "incorrect name stored"
    );

    assert_ne!(
        child_dir,
        stored.base_path(),
        "container path should change"
    );

    if properties.name.is_ascii() {
        assert_eq!(
            &properties.name,
            stored.base_path().file_name().unwrap().to_str().unwrap(),
            "folder name and name should agree"
        );
    }

    assert!(
        Path::exists(stored.base_path()),
        "child folder should be renamed"
    );
}
