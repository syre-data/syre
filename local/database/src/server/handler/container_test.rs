use super::*;
use crate::command::{ContainerCommand, GraphCommand, ProjectCommand};
use crate::error::Result;
use dev_utils::fs::TempDir;
use fake::faker::lorem::raw::Word;
use fake::locales::EN;
use fake::Fake;
use thot_core::project::Container as CoreContainer;
use thot_local::project::resources::Container as LocalContainer;
use thot_local::project::{container, project};

#[test]
fn database_command_update_container_properties_should_work() {
    // setup
    let mut dir = TempDir::new().expect("could not create new temp dir");
    let data_dir = dir.mkdir().unwrap();
    let pid = project::init(dir.path()).unwrap();
    let _rid = container::init(&data_dir).expect("could not init `Container`");
    let mut db = Database::new();
    let _project = db.handle_command_project(ProjectCommand::Load(dir.path().into()));
    let container = db.handle_command_graph(GraphCommand::Load(pid));

    let container: Result<CoreContainer> =
        serde_json::from_value(container).expect("could not contvert JsValue to `Container`");

    let container = container.expect("`LoadContainer` should work");
    let mut properties = container.properties.clone();
    let name = Word(EN).fake();
    properties.name = name;

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

    let saved = LocalContainer::load_from(dir.path()).expect("could not load `Container`");
    assert_eq!(
        properties.name, saved.properties.name,
        "incorrect name persisted"
    );
}
