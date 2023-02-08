//! Dev utils for database.
use crate::db::resources::standard_properties::StandardProperties;
use fake::faker::lorem::raw::{Word, Words};
use fake::locales::EN;
use fake::Fake;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Create a fake StandardProperties.
///
/// # Arguments
/// + `name_none`: `None` if `name` is allowed to be `None` or `Some`, chosen randomly.
///     `Some(false)` to force `name` to be `None` or `Some(true)` to force `name` to have a value.
/// + `kind`: `None` if `kind` is allowed to be `None` or `Some`, chosen randomly.
///     `Some(false)` to force `kind` to be `None` or `Some(true)` to force `kind` to have a value.
pub fn mock_props(name_none: Option<bool>, kind_none: Option<bool>) -> StandardProperties {
    // tags
    let tag_words: Vec<String> = Words(EN, 1..20).fake();
    let mut tags = HashSet::new();
    for t in tag_words {
        tags.insert(t);
    }

    // metadata
    let mut md = HashMap::new();
    md.insert(
        String::from("str_val"),
        (
            Value::String(Word(EN).fake::<String>()),
            rand::random::<bool>(),
        ),
    );

    md.insert(
        String::from("int_val"),
        (
            Value::Number(rand::random::<i32>().into()),
            rand::random::<bool>(),
        ),
    );

    md.insert(
        String::from("bool_val"),
        (Value::Bool(rand::random::<bool>()), rand::random::<bool>()),
    );

    // props
    StandardProperties {
        name: fake_opt_str(name_none),
        kind: fake_opt_str(kind_none),
        tags,
        metadata: md,
    }
}

/// Create a fake StandardProperties.
///
/// # Arguments
/// + `none`: `None` if allowed to be `None` or `Some`, chosen randomly.
///     `Some(false)` to force to be `None` or `Some(true)` to force to have a value.
fn fake_opt_str(none: Option<bool>) -> Option<String> {
    let opt_val = match none {
        None => rand::random(),
        Some(true) => true,
        Some(false) => false,
    };

    match opt_val {
        true => Some(Word(EN).fake::<String>()),
        false => None,
    }
}
