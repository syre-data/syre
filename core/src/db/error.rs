//! Database related errors.
use crate::types::ResourceId;

#[derive(Debug)]
pub enum Error {
    /// A resource with the given `ResourceId` does not exist.
    DoesNotExist(ResourceId),

    /// A resource with the given `ResourceId` already exists.
    AlreadyExists(ResourceId),

    /// NO matches found when expected.
    NoMatches,

    /// Multiple matches are found when only one or none are expected.
    MultipleMatches,
}

#[cfg(test)]
#[path = "./error_test.rs"]
mod error_test;
