use super::*;
use crate::dev_utils::resource_path;
use crate::project::asset::Asset as PrjAsset;
use crate::project::container::Container;
use crate::types::{ResourceId, ResourcePath};
use fake::faker::filesystem::raw::FileName;
use fake::locales::EN;
use fake::Fake;
use std::collections::{HashMap, HashSet};

#[test]
fn from_project_asset_should_work() {
    // setup
    let prj = PrjAsset::new(resource_path(Some("py")));
    let container = Container::new();

    // test
    let db = Asset::from(prj.clone(), container.rid.clone());
    assert_eq!(prj.rid, db.rid, "resource ids should match");
}

#[test]
fn bucket_should_work() {
    let a0 = mock_asset(None);
    assert_eq!(
        Some(PathBuf::from("")),
        a0.bucket(),
        "asset should not be in bucket"
    );

    let a1 = mock_asset(Some("tmp"));
    assert_eq!(Some(PathBuf::from("tmp")), a1.bucket(), "incorrect bucket")
}

#[test]
fn of_root_should_work() {
    let asset = mock_asset(None);
    assert!(asset.of_root(), "asset should be of root");
}

#[test]
fn in_bucket_should_work() {
    // setup
    let asset = mock_asset(Some("tmp"));
    let bucket = asset.bucket().expect("should not be in root");

    // test
    assert_eq!(true, asset.in_bucket(&bucket), "should be in own bucket");

    let parent = bucket.parent().unwrap();
    let parent = parent.to_path_buf();
    assert_eq!(true, asset.in_bucket(&parent), "should be in parent bucket");

    let mut sibling = parent.clone();
    sibling.push("sib");
    assert_eq!(
        false,
        asset.in_bucket(&sibling),
        "should not be in sibling bucket"
    );

    let mut child = bucket.clone();
    child.push("tmp");
    assert_eq!(
        false,
        asset.in_bucket(&child),
        "should not be in child bucket"
    );
}

#[test]
fn of_bucket_should_work() {
    // setup
    let asset = mock_asset(Some("tmp"));
    let Some(bucket) = asset.bucket() else {
        panic!("should not be in root");
    };

    // test
    assert_eq!(true, asset.of_bucket(&bucket), "should be of own bucket");

    let parent = bucket.parent().expect("bucket parent should work");
    let parent = parent.to_path_buf();
    assert_eq!(
        false,
        asset.of_bucket(&parent),
        "should not be of parent bucket"
    );

    let mut sibling = parent.clone();
    sibling.push("sib");
    assert_eq!(
        false,
        asset.of_bucket(&sibling),
        "should not be of sibling bucket"
    );

    let mut child = bucket.clone();
    child.push("tmp");
    assert_eq!(
        false,
        asset.of_bucket(&child),
        "should not be of child bucket"
    );
}

// ***************
// *** helpers ***
// ***************

/// Create a [`ResourcePath`].
///
/// # Arguments
/// 1. `parent`: Parent path.
fn mock_path(parent: Option<&str>) -> PathBuf {
    let mut f_name = PathBuf::from(FileName(EN).fake::<String>());
    f_name.set_extension("py");

    if let Some(parent) = parent {
        let mut parent = PathBuf::from(parent);
        parent.push(f_name);
        parent
    } else {
        f_name
    }
}
/// Creates a new [`Asset`].
///
/// # Arguments
/// 1. `bucket`: The `Asset`'s `bucket`.
fn mock_asset(bucket: Option<&str>) -> Asset {
    // setup
    let properties = StandardProperties {
        name: None,
        kind: None,
        tags: HashSet::new(),
        metadata: HashMap::new(),
    };

    let path = ResourcePath::new(mock_path(bucket)).expect("`ResourcePath` should work");

    Asset {
        rid: ResourceId::new(),
        properties,
        path,
        parent: None,
    }
}
