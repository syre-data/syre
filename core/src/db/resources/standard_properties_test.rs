use super::*;
use crate::project::standard_properties::StandardProperties;
use serde_json::json;

#[cfg(feature = "serde")]
use serde::Deserialize;

#[test]
fn from_project_standard_properties_should_work() {
    let prj = PrjStdProps::new();
    let db: StandardProperties = prj.clone().into();
    assert_eq!(prj.name, db.name, "names should match");
    assert_eq!(prj.kind, db.kind, "kinds should match");
}

#[cfg(feature = "serde")]
#[test]
fn deserialize_metadata_should_work() {
    // setup
    #[derive(Deserialize)]
    struct MDVal {
        #[serde(deserialize_with = "deserialize_metadata")]
        pub md: Metadata,
    }

    let json = r#"{"md": {
        "bool_val": true, 
        "num_val": 1, 
        "obj_val": {
            "str_val": "test"
        }
    }}"#;

    // test
    let md_val: MDVal = serde_json::from_str(json).expect("deserialization should work");
    let md = md_val.md;

    // bool val
    let bool_val = md.get("bool_val").expect("boolean value should exist");
    assert_eq!(
        &(json!(true), false),
        bool_val,
        "bool value should be correct"
    );

    // num val
    let num_val = md.get("num_val").expect("number value should exist");
    assert_eq!(&(json!(1), false), num_val, "num value should be correct");

    // obj val
    let obj_val = md.get("obj_val").expect("object value should exist");
    assert_eq!(
        &(json!({"str_val": "test"}), false),
        obj_val,
        "object value should be correct"
    );
}
