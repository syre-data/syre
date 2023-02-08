use super::*;
use dev_utils::path::resource_path::resource_path;

// *************
// *** Asset ***
// *************

#[test]
fn local_asset_new_should_work() {
    let file = resource_path(Some("py"));
    let asset = Asset::new(file.clone()).expect("new should work");
}

// **************
// *** Assets ***
// **************

#[test]
fn assets_insert_asset_should_work() {
    // setup
    let mut assets = Assets::new();
    let a0 = Asset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    let rid0 = a0.rid.clone();

    let a1 = Asset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    let mut a0r = Asset::new(resource_path(Some("py"))).expect("new `Asset` should work");
    a0r.rid = rid0;

    // test
    let a0_res = assets.insert_asset(a0).expect("inert asset should work");
    assert_eq!(1, assets.len(), "assets not inserted correctly");
    assert!(a0_res.is_none(), "asset already existed");

    let a1_res = assets.insert_asset(a1).expect("inert asset should work");
    assert_eq!(2, assets.len(), "assets not inserted correctly");
    assert!(a1_res.is_none(), "asset already existed");

    let a0r_res = assets.insert_asset(a0r).expect("inert asset should work");
    assert_eq!(2, assets.len(), "assets not inserted correctly");
    assert!(a0r_res.is_some(), "asset did not exist");
}

#[test]
#[should_panic(expected = "FileAlreadyAsset")]
fn assets_insert_asset_for_file_that_already_exists_should_error() {
    // setup
    let mut assets = Assets::new();
    let path = resource_path(Some("py"));
    let a0 = Asset::new(path.clone()).expect("new `Asset` should work");
    let a1 = Asset::new(path).expect("new `Asset` should work");

    // test
    assets.insert_asset(a0).expect("insert asset should work");
    assets.insert_asset(a1).unwrap();
}

#[test]
fn assets_get_path_should_work() {
    // setup
    let mut assets = Assets::new();
    let p0 = resource_path(Some("py"));
    let a0 = Asset::new(p0.clone()).expect("new `Asset` should work");
    assets.insert_asset(a0);

    // test
    let found = assets.get_path(&p0);
    assert!(found.is_some(), "should find inserted `Asset`");

    let found = assets.get_path(&resource_path(Some("py")));
    assert!(
        found.is_none(),
        "should not find `Asset`s that were not inserted"
    );
}
