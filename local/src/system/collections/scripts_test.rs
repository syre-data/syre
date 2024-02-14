use super::*;
use syre_core::project::Script as CoreScript;

#[test]
fn scripts_contains_path_should_work() {
    // setup
    let path = PathBuf::from("script.py");
    let script = CoreScript::from_path(path.clone()).unwrap();
    let rid = script.rid.clone();

    let mut scripts = Scripts::load().unwrap();
    scripts.insert(rid.clone(), script);

    // test
    assert!(
        scripts.contains_path(&path),
        "scripts should contain script"
    );

    assert_eq!(
        false,
        scripts.contains_path("another_script.py"),
        "scripts should not contain random path"
    );

    // clean up
    scripts.remove(&rid);
    scripts
        .save()
        .expect("could not save `Scripts` during clean up");
}

#[test]
fn scripts_by_path_should_work() {
    // setup
    let path = PathBuf::from("script.py");
    let script = CoreScript::from_path(path.clone()).unwrap();
    let rid = script.rid.clone();

    let mut scripts = Scripts::load().unwrap();
    scripts.insert(script.rid.clone(), script);

    // test
    // inserted script
    let found = scripts.by_path(&path).unwrap();
    assert_eq!(&rid, &found.rid);

    // not inserted script
    let rand = scripts.by_path("another_script.py");
    assert!(rand.is_none());
}
