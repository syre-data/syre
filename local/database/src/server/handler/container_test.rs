use super::*;
use crate::command::ContainerCommand;
use crate::error::Result;
use dev_utils::fs::TempDir;
use fake::faker::lorem::raw::Word;
use fake::locales::EN;
use fake::Fake;
use thot_core::project::Container as CoreContainer;
use thot_local::project::container;
use thot_local::project::resources::Container as LocalContainer;

#[test]
fn database_command_update_container_properties_should_work() {
    // setup
    let _dir = TempDir::new().expect("could not create new temp dir");
    let _rid = container::init(_dir.path()).expect("could not init `Container`");
    let mut db = Database::new();
    let container = db.handle_command_container(ContainerCommand::load(_dir.path().to_path_buf()));

    let container: Result<CoreContainer> =
        serde_json::from_value(container).expect("could not contvert JsValue to `Container`");

    let container = container.expect("`LoadContainer` should work");
    let mut properties = container.properties.clone();
    let name = Word(EN).fake();
    properties.name = Some(name);

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

        let stored = stored.lock().expect("could not lock `Container`");
        assert_eq!(
            properties.name, stored.properties.name,
            "incorrect name stored"
        );

        // ensure persisted container updated
        db.store.remove_container(&container.rid);
    }

    let saved = LocalContainer::load_from(_dir.path()).expect("could not load `Container`");
    assert_eq!(
        properties.name, saved.properties.name,
        "incorrect name persisted"
    );
}

#[test]
fn database_command_update_asset_should_work() {
    todo!();
}

#[test]
fn database_add_assets_should_work() {
    todo!();
}
