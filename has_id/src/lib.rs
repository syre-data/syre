//! Indicates an object has a unique id.
use std::hash::Hash;

// Re-exports
#[cfg(feature = "derive")]
use has_id_derive;

#[cfg(feature = "derive")]
pub use has_id_derive::HasId;

#[cfg(feature = "derive")]
pub use has_id_derive::HasIdMut;

#[cfg(all(feature = "derive", feature = "serde"))]
pub use has_id_derive::HasIdSerde;

/// Indicates an object has a unique id.
pub trait HasId {
    type Id: Hash + Eq;

    fn id(&self) -> &Self::Id;
}

/// Indicates an object has a unique id.
#[cfg(feature = "serde")]
pub trait HasIdSerde<'de> {
    type Id: Hash + Eq + Clone + serde::Serialize + serde::Deserialize<'de>;

    fn id(&self) -> &Self::Id;
}

/// Indicates that the id can be mutated.
pub trait HasIdMut: HasId {
    fn id_mut(&mut self) -> &mut Self::Id;
}

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
