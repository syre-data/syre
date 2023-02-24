//! Dev utils for database.
use crate::db::{Resource, StandardResource};
use crate::project::StandardProperties;
use crate::types::ResourceId;
use fake::faker::lorem::raw::{Word, Words};
use fake::locales::EN;
use fake::Fake;
use has_id::HasId;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::Deserialize;

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

    // metadata
    let mut md = HashMap::new();
    md.insert(
        String::from("str_val"),
        Value::String(Word(EN).fake::<String>()),
    );

    md.insert(
        String::from("int_val"),
        Value::Number(rand::random::<i32>().into()),
    );

    md.insert(
        String::from("bool_val"),
        Value::Bool(rand::random::<bool>()),
    );

    // props
    let mut props = StandardProperties::default();
    props.name = fake_opt_str(name_none);
    props.kind = fake_opt_str(kind_none);
    props.tags = tag_words;
    props.metadata = md;

    props
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

// -----------------------
// --- Standard Object ---
// -----------------------

#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(HasId, Clone, Debug, PartialEq, Eq)]
pub struct StdObj {
    #[id]
    pub rid: ResourceId,
    pub props: StandardProperties,
}

impl StdObj {
    /// Create a new StdObj.
    ///
    /// # Arguments
    /// + `name_none`: `None` if `name` is allowed to be `None` or `Some`, chosen randomly.
    ///     `Some(false)` to force `name` to be `None` or `Some(true)` to force `name` to have a value.
    /// + `kind`: `None` if `kind` is allowed to be `None` or `Some`, chosen randomly.
    ///     `Some(false)` to force `kind` to be `None` or `Some(true)` to force `kind` to have a value.
    pub fn new(name_none: Option<bool>, kind_none: Option<bool>) -> StdObj {
        Self {
            rid: ResourceId::new(),
            props: mock_props(name_none, kind_none),
        }
    }
}

impl Hash for StdObj {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rid.hash(state);
    }
}

impl Resource for StdObj {}
impl StandardResource for StdObj {
    fn properties(&self) -> &StandardProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut StandardProperties {
        &mut self.props
    }
}
