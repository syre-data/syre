//! Common use functions.
use crate::constants::*;
use std::path::PathBuf;

/// Creates a root drive id with the given metalevel.
/// Has the form `ROOT_DRIVE_ID[metalevel]:`.
pub fn root_drive_with_metalevel(metalevel: usize) -> PathBuf {
    let root_drive = format!("{ROOT_DRIVE_ID}[{metalevel}]:");
    PathBuf::from(root_drive)
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;
