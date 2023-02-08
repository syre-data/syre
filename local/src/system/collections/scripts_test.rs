use super::*;
use dev_utils::path::resource_path::resource_path;
use thot_core::project::Script as CoreScript;

#[test]
fn scripts_contains_path_should_work() {
    // setup
    let path = resource_path(Some("py"));
    let script = CoreScript::new(path.clone()).expect("creating script should work");
    let rid = script.rid.clone();

    let mut scripts = Scripts::load().expect("could not load `Scripts`");
    scripts.insert(rid.clone(), script);

    // test
    assert!(
        scripts.contains_path(&path),
        "scripts should contain script"
    );

    assert_eq!(
        false,
        scripts.contains_path(&resource_path(Some("py"))),
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
    let path = resource_path(Some("py"));
    let script = CoreScript::new(path.clone()).expect("creating script should work");
    let rid = script.rid.clone();

    let mut scripts = Scripts::load().expect("could not load `Scripts`");
    scripts.insert(script.rid.clone(), script);

    // test
    // inserted script
    let found = scripts.by_path(&path);
    assert_eq!(1, found.len(), "script should be found");

    let found = found.get(&rid).expect("could not unwrap found `Script`");
    assert_eq!(&rid, &found.rid, "found script should be correct");

    // not inserted script
    let rand = scripts.by_path(&resource_path(Some("py")));
    assert_eq!(0, rand.len(), "script should not be found");
}
