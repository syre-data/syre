//! Indicate an object has a serializable id.
use serde::{de::Deserialize, Serialize};
use std::hash::Hash;

// TODO[l]: Make a supertrait of `HasId`.
// Would require adding additional subtrait requirements to
// `HasId::Id`.
/// Indicates an object has a unique id.
pub trait HasIdSerde<'de> {
    type Id: Hash + Eq + Clone + Serialize + Deserialize<'de>;

    fn id(&self) -> &Self::Id;
}
