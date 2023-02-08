use super::*;
use fake::faker::filesystem::raw::FilePath;
use fake::locales::EN;
use fake::Fake;
use std::path::PathBuf;

// *********************
// *** System Script ***
// *********************

// ***************
// *** Scripts ***
// ***************

// @todo: Not sure if `Scripts` still needs to exist.

// #[test]
// fn scripts_contains_path_should_work() {
//     // setup
//     let path = script_path();
//     let script = CoreScript::new(path.clone()).expect("creating script should work");

//     let mut scripts = Scripts::new();
//     scripts.push(script);

//     // test
//     assert!(
//         scripts.contains_path(&path),
//         "scripts should contain script"
//     );

//     assert_eq!(
//         false,
//         scripts.contains_path(&script_path()),
//         "scripts should not contain random path"
//     );
// }

// #[test]
// fn scripts_by_path_should_work() {
//     // setup
//     let path = script_path();
//     let script = CoreScript::new(path.clone()).expect("creating script should work");
//     let rid = script.rid.clone();

//     let mut scripts = Scripts::new();
//     scripts.push(script);

//     // test
//     // inserted script
//     let found = scripts.by_path(&path);
//     assert!(found.is_some(), "script should be found");

//     let found = found.unwrap();
//     assert_eq!(&rid, &found.rid, "found script should be correct");

//     // not inserted script
//     let rand = scripts.by_path(&script_path());
//     assert!(rand.is_none(), "script should not be found");
// }

// ************************
// *** helper functions ***
// ************************

fn script_path() -> ResourcePath {
    let path = PathBuf::from(FilePath(EN).fake::<String>());
    ResourcePath::new(path).expect("creating resource path should work")
}
