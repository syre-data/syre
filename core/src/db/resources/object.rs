//! Database object functionality.
use super::standard_properties::StandardProperties;
use crate::types::ResourceId;
use has_id::{HasId, HasIdMut};
use std::hash::Hash;

// @todo: Rename to `Resource`.
/// Functionality for database objects.
pub trait Object: HasId<Id = ResourceId> + HasIdMut + Clone + Hash + PartialEq + Eq {}

// @todo: Rename to `StandardResource`.
/// Functionality for standard objects.
pub trait StandardObject: Object {
    /// Retrieve a reference to the [`StandardProperties`] of the object.
    fn properties(&self) -> &StandardProperties;

    /// Retrieve a mutable reference to the [`StandardProperties`] of the object.
    fn properties_mut(&mut self) -> &mut StandardProperties;
}

#[cfg(test)]
#[path = "./object_test.rs"]
mod object_test;
