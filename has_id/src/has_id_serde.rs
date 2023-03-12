//! Indicate an object has a serializable id.
use std::hash::Hash;

/// Indicates an object has a unique id.
pub trait HasIdSerde<'de> {
    type Id: Hash + Eq + Clone + serde::Serialize + serde::Deserialize<'de>;

    fn id(&self) -> &Self::Id;
}

#[cfg(test)]
#[path = "./has_id_serde_test.rs"]
mod has_id_serde_test;
