use super::*;

#[test]
fn asset_bucket_should_work() {
    // setup
    let abs_file = PathBuf::from("asset");
    let rel_file = PathBuf::from("parent/asset");

    let abs_asset = Asset::new(abs_file.clone());
    let rel_asset = Asset::new(rel_file.clone());

    // test
    assert_eq!(None, abs_asset.bucket(), "asset should not have bucket");

    let bucket = rel_asset.bucket().unwrap();
    assert_eq!(rel_file.parent().unwrap(), bucket);
}
