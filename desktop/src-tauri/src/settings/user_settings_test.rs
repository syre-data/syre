use super::*;
use std::path::PathBuf;
use thot_core::types::ResourceId;

#[test]
fn user_path_should_be_correct() {
    let uid = ResourceId::new();

    let mut expected = PathBuf::from(uid.to_string());
    expected.set_extension("json");

    assert_eq!(
        expected,
        UserSettings::rel_path(uid),
        "incorrect relative path"
    );
}
