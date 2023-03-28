//! Indicate an object has a serializable id.
use serde::{de::Deserialize, Serialize};
use std::hash::Hash;

/// Indicates an object has a unique id.
pub trait HasIdSerde<'de> {
    type Id: Hash + Eq + Clone + Serialize + Deserialize<'de>;

    fn id(&self) -> &Self::Id;
}

#[cfg(test)]
#[path = "./has_id_serde_test.rs"]
mod has_id_serde_test;
