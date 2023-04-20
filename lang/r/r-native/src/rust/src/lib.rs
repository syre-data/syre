use extendr_api::prelude::*;
use std::path::{Path, PathBuf};
use thot_core::types::ResourcePath;
use thot_local::project::{project, resources::Asset as LocalAsset};

/// Gets the Project path given a path.
/// Returns `NULL` if the path is not in a project.
///
/// @param path Path to get the Project root of.
#[extendr]
fn project_resource_root_path(path: String) -> Nullable<String> {
    match project::project_resource_root_path(&Path::new(&path)) {
        Ok(project) => Nullable::NotNull(project.as_os_str().to_str().unwrap().to_string()),
        _ => Nullable::Null,
    }
}

/// Creates a new Asset.
///
/// @param path Path of the Asset's file.
#[extendr]
fn new_asset(path: String) -> String {
    let path = PathBuf::from(&path);
    let path = ResourcePath::new(path).expect("could not create path");
    let asset = LocalAsset::new(path).expect("could not create Asset");
    let asset = serde_json::to_value(asset).expect("could not convert Asset to JsValue");
    serde_json::to_string(&asset).expect("could not convert Asset to string")
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod thot;
    fn project_resource_root_path;
    fn new_asset;
}
