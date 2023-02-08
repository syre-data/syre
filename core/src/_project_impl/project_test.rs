use super::*;

// ***************
// *** Project ***
// ***************

#[test]
fn project_new_w_defaults() {
    let p_name = String::from("test");
    let prj = Project::new(p_name.as_str());

    assert_eq!(p_name, prj.name, "incorrect project name");
    assert_eq!(None, prj.description, "incorrect default description");
    assert_eq!(None, prj.data_root, "incorrect default data path");
    assert_eq!(None, prj.universal_root, "incorrect defualt universal root");
    assert_eq!(None, prj.analysis_root, "incorrect default analysis root");
    assert_eq!(0, prj.meta_level, "incorrect default meta level");
}
