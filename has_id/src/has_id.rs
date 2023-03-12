//! Basic `HasId`.
use std::hash::Hash;

/// Indicates an object has a unique id.
pub trait HasId {
    type Id: Hash + Eq;

    fn id(&self) -> &Self::Id;
}

#[cfg(test)]
#[path = "./has_id_test.rs"]
mod has_id_test;
