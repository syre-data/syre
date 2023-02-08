use super::*;

#[test]
fn qualifier_should_be_com() {
    assert_eq!("com", Identifier::qualifier(), "qualifier should be `com`");
}

#[test]
fn organization_should_be_thot() {
    assert_eq!(
        "Thot",
        Identifier::organization(),
        "organization should be `Thot`"
    );
}

#[test]
fn application_should_be_thot_core() {
    assert_eq!(
        "Thot Core",
        Identifier::application(),
        "application should be `Thot Core`"
    );
}
