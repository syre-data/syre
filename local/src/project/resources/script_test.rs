use super::*;
use dev_utils::path::resource_path::resource_path;

// ********************
// *** Local Script ***
// ********************

#[test]
fn script_new_should_work() {
    let path = resource_path(Some("py"));
    let script = Script::new(path.clone()).expect("creating script should work");
    assert_eq!(&path, &script.path, "script's path should be correct");
}
