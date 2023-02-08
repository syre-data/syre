use super::*;

#[test]
fn new_should_work() {
    TempDir::new().expect("new should work");
}

#[test]
fn mkdir_should_work() {
    let mut dir = TempDir::new().expect("new should work");
    let c = dir.mkdir().expect("mkdir should work");
    assert!(dir.children.contains_key(&c), "child should be in children");
}

#[test]
fn mkfile_should_work() {
    let mut dir = TempDir::new().expect("new should work");
    let path = dir.mkfile().expect("mkfile should work");
    assert!(
        dir.files.contains_key(&path),
        "file should be contained in files"
    );
}

#[test]
fn mkfile_with_extension_should_work() {
    let mut dir = TempDir::new().expect("new should work");
    let path = dir
        .mkfile_with_extension(".py")
        .expect("mkfile_with_extension should work");

    assert!(
        dir.files.contains_key(&path),
        "file should be contained in files"
    );
}
