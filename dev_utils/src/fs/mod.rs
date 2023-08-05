//! File system utilities.
//!
//! # See also
//! + For similar functionality that does not create file system side effects
//! see the [`path`](crate::path) module.
pub mod temp_dir;
pub mod temp_file;

// Re-exports
pub use temp_dir::TempDir;
