use super::*;

#[test]
fn root_drive_with_metalevel_should_work() {
    // setup
    let metalevel: usize = rand::random();
    let mut expected = String::from(ROOT_DRIVE_ID);
    let ml = format!("[{metalevel}]:");
    expected.push_str(&ml);
    let expected = PathBuf::from(expected);

    // test
    let path = root_drive_with_metalevel(metalevel);
    assert_eq!(expected, path, "path should be correct");
}
