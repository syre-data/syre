use super::*;
use std::path::PathBuf;

#[test]
fn path_distance_should_work() {
    let a = PathBuf::from("a");
    assert_eq!(utils::path_distance(&a, &a), 0 as usize);

    let b = PathBuf::from("b");
    assert_eq!(utils::path_distance(&a, &b), 1 as usize);
}
