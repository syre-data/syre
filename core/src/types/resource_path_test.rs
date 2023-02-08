use super::*;
use crate::common::root_drive_with_metalevel;
use crate::constants::ROOT_DRIVE_ID;
use fake::faker::filesystem::raw::FilePath;
use fake::locales::EN;
use fake::Fake;
use std::assert_matches::assert_matches;

#[test]
fn resource_path_is_absolute_path_should_work() {
    // setup
    let fp = FilePath(EN).fake::<String>();
    let path = Path::new(&fp);

    // test
    assert_eq!(
        true,
        ResourcePath::is_absolute(path),
        "is absolute should be true"
    );

    assert_eq!(
        false,
        ResourcePath::is_root(path),
        "is root should be false"
    );

    assert_eq!(
        false,
        ResourcePath::is_relative(path),
        "is relative should be false"
    );
}

#[test]
fn resource_path_is_relative_path_should_work() {
    // setup
    let fp = FilePath(EN).fake::<String>();
    let mut path = String::from("..");
    path.push_str(&fp);
    let path = Path::new(&path); // make relative

    // test
    assert_eq!(
        true,
        ResourcePath::is_relative(path),
        "is relative should be true"
    );

    assert_eq!(
        false,
        ResourcePath::is_root(path),
        "is root should be false"
    );

    assert_eq!(
        false,
        ResourcePath::is_absolute(path),
        "is absolute should be false"
    );
}

#[test]
fn resource_path_is_root_path_should_work() {
    let fp = FilePath(EN).fake::<String>();
    let mut path = String::from(ROOT_DRIVE_ID);
    path.push_str(&fp);
    let path = Path::new(&path);

    // test
    assert_eq!(true, ResourcePath::is_root(path), "is root should be true");
    assert_eq!(
        false,
        ResourcePath::is_absolute(path),
        "is absolute should be false"
    );
    assert_eq!(
        false,
        ResourcePath::is_relative(path),
        "is relative should be false"
    );
}

#[test]
fn resource_path_new_with_absolute_path_should_work() {
    // setup
    let fp = FilePath(EN).fake::<String>();
    let name = PathBuf::from(fp);
    let expected = ResourcePath::Absolute(name.clone());

    // test
    let path = ResourcePath::new(name.clone()).expect("creating resource path should work");
    assert_matches!(path, ResourcePath::Absolute(_), "incorrect path type");
    assert_eq!(expected, path, "incorrect path");
}

#[test]
fn resource_path_new_with_relative_path_should_work() {
    // setup
    let fp = FilePath(EN).fake::<String>();
    let mut name = String::from("..");
    name.push_str(&fp);

    let name = PathBuf::from(name); // make relative
    let expected = ResourcePath::Relative(name.clone());

    // test
    let path = ResourcePath::new(name.clone()).expect("creating resource path should work");
    assert_matches!(path, ResourcePath::Relative(_), "incorrect path type");
    assert_eq!(expected, path, "incorrect path");
}

#[test]
fn resource_path_new_with_default_root_path_should_work() {
    // setup
    let fp = FilePath(EN).fake::<String>();
    let name = format!("{ROOT_DRIVE_ID}:{fp}");
    let name = PathBuf::from(name);
    let fp = PathBuf::from(&fp[1..]);
    let expected = ResourcePath::Root(fp, 0);

    // test
    let path = ResourcePath::new(name.clone()).expect("creating resource path should work");
    assert_matches!(path, ResourcePath::Root(_, _), "incorrect path type");
    assert_eq!(expected, path, "incorrect path");
}

#[test]
fn resource_path_new_with_root_path_metalevel_should_work() {
    // setup
    let fp = FilePath(EN).fake::<String>();
    let fp = &fp[1..];
    let metalevel: usize = rand::random();
    let mut name = root_drive_with_metalevel(metalevel);
    name.push(fp);
    let fp = PathBuf::from(fp);

    // test
    let path = ResourcePath::new(name).expect("creating resource path should work");
    assert_matches!(&path, &ResourcePath::Root(_, _), "incorrect path type");
    if let ResourcePath::Root(p, ml) = path {
        assert_eq!(fp, p, "incorrect path");
        assert_eq!(metalevel, ml, "incorrect metalevel");
    }
}

#[test]
#[should_panic(expected = "CouldNotParseMetalevel")]
fn resource_path_root_path_with_invalid_metalevel_should_error() {
    // setup
    let fp = FilePath(EN).fake::<String>();
    let metalevel: usize = rand::random();
    let name = root_drive_with_metalevel(metalevel);
    let mut name = String::from(name.to_str().unwrap());
    name.insert_str(name.len() - 2, ".1"); // add decimal to end of metalevel
    name.push_str(&fp);

    let name = PathBuf::from(name);

    // test
    ResourcePath::new(name).unwrap();
}

#[test]
fn resource_path_absolute_into_path_buf_should_work() {
    let fp = FilePath(EN).fake::<String>();
    let expected = PathBuf::from(fp);
    let rp = ResourcePath::Absolute(expected.clone());

    let pb: PathBuf = rp.into();
    assert_eq!(expected, pb, "conversion to path buf should work");
}

#[test]
fn resource_path_relative_into_path_buf_should_work() {
    let fp = FilePath(EN).fake::<String>();
    let mut expected = PathBuf::from("..");
    expected.push(&fp);
    let rp = ResourcePath::Relative(expected.clone());

    let pb: PathBuf = rp.into();
    assert_eq!(expected, pb, "conversion to path buf should work");
}

#[test]
fn resource_path_root_into_path_buf_should_work() {
    let fp = FilePath(EN).fake::<String>();
    let mut expected = PathBuf::from(ROOT_DRIVE_ID);
    expected.push(&fp);
    let rp = ResourcePath::Root(expected.clone(), 0);

    let pb: PathBuf = rp.into();
    assert_eq!(expected, pb, "conversion to path buf should work");
}
