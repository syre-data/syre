use super::*;
use uuid::Uuid;

#[test]
fn resource_id_new_should_work() {
    let _rid: ResourceId = ResourceId::new();
}

#[test]
fn resource_id_from_uuid_should_work() {
    let uid = Uuid::new_v4();
    let rid: ResourceId = ResourceId::from(uid.clone());

    assert_eq!(uid, *rid, "uid and rid should match");
}

#[test]
fn resource_id_into_uuid_should_work() {
    let rid: ResourceId = ResourceId::new();
    let uid = rid.0.clone();
    let ruid: Uuid = rid.into();

    assert_eq!(uid, ruid, "resource id should be transformed into uuid");
}
