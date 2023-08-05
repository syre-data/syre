use super::*;
use crate::common::{project_file_of, project_settings_file_of};
use std::path::Path;

// ***************
// *** Project ***
// ***************

#[test]
fn project_new_w_defaults() {
    let p_name = String::from("test");
    let project = Project::new(p_name.as_str()).expect("new should work");

    assert_eq!(p_name, project.name, "incorrect project name");
    assert_eq!(None, project.description, "incorrect default description");
    assert_eq!(None, project.data_root, "incorrect default data path");
    assert_eq!(
        None, project.universal_root,
        "incorrect defualt universal root"
    );
    assert_eq!(
        None, project.analysis_root,
        "incorrect default analysis root"
    );
    assert_eq!(0, project.meta_level, "incorrect default meta level");
}

#[test]
fn project_rel_path_should_be_correct() {
    let rel_path = Project::rel_path().expect("rel_path should not error");

    assert_eq!(
        project_file_of(Path::new("")),
        rel_path,
        "realtive path is incorrect"
    );
}

// ************************
// *** Project Settings ***
// ************************

#[test]
fn project_settings_new_should_work() {
    let _sets = ProjectSettings::new();
}

#[test]
fn project_settings_rel_path_should_be_correct() {
    let rel_path = ProjectSettings::rel_path().expect("rel_path should not error");

    assert_eq!(
        project_settings_file_of(Path::new("")),
        rel_path,
        "incorrect relative path"
    );
}

#[test]
fn project_settings_priority_should_be_correct() {
    let sets = ProjectSettings::new();
    assert_eq!(
        SettingsPriority::Local,
        sets.priority(),
        "incorrect priority"
    );
}
