//! Database object functionality.
use crate::types::ResourceId;
use has_id::HasId;
use std::hash::Hash;

/// Functionality for database objects.
pub trait Resource: HasId<Id = ResourceId> + Hash + Clone + PartialEq + Eq {}
