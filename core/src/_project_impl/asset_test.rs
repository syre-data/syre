use super::*;
use crate::dev_utils::resource_path;
use crate::types::ResourcePath;

#[test]
fn asset_new_should_work() {
    let file = resource_path(Some("py"));
    let asset: Asset = Asset::new(file.clone());

    assert_eq!(file, asset.path, "file should be initialized");
}

#[test]
fn asset_bucket_should_work() {
    // setup
    let abs_file = resource_path(Some("py"));
    let rel_file = abs_file.as_path();
    let rel_file = rel_file
        .strip_prefix("/")
        .expect("make path relative should work");

    let rel_file =
        ResourcePath::new(rel_file.to_path_buf()).expect("new `ResourcePath` should work");

    let abs_asset = Asset::new(abs_file);
    let rel_asset = Asset::new(rel_file.clone());

    // test
    assert_eq!(None, abs_asset.bucket(), "asset should not have bucket");

    let Some(bucket) = rel_asset.bucket() else {
        panic!("asset should have bucket");
    };

    assert_eq!(
        rel_file.as_path().parent().expect("parent should exist"),
        bucket
    );
}
