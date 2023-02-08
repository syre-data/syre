//! Path related utilities.
//!
//! # See also
//! + For utilities that have side effects in the file system, see the [`fs`](crate::fs) module.
pub mod resource_path;

// Re-exports

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
