//! Indicates an object has a unique id.
pub mod has_id;

// TODO: Remove.
pub mod has_id_mut;

#[cfg(feature = "serde")]
pub mod has_id_serde;

// Re-exports
pub use has_id::HasId;
pub use has_id_mut::HasIdMut;

#[cfg(feature = "serde")]
pub use has_id_serde::HasIdSerde;

#[cfg(feature = "derive")]
pub use has_id_derive::HasId;

#[cfg(all(feature = "derive", feature = "serde"))]
pub use has_id_derive::HasIdSerde;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
