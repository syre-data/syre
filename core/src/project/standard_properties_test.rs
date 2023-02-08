use super::*;

#[test]
fn new_should_work() {
    let res = StandardProperties::new();

    // name
    assert_eq!(None, res.name, "name should be None");

    // kind
    assert_eq!(None, res.kind, "kind should be None");

    // description
    assert_eq!(None, res.description, "description should be None");

    // tags
    assert_eq!(0, res.tags.len(), "tags should be empty");

    // metadata
    assert_eq!(0, res.metadata.len(), "metadata should be empty");
}

#[test]
fn default_should_work() {
    let res = StandardProperties::default();

    // name
    assert_eq!(None, res.name, "name should be None");

    // kind
    assert_eq!(None, res.kind, "kind should be None");

    // description
    assert_eq!(None, res.description, "description should be None");

    // tags
    assert_eq!(0, res.tags.len(), "tags should be empty");

    // metadata
    assert_eq!(0, res.metadata.len(), "metadata should be empty");
}
