use super::*;
use std::path::PathBuf;
use thot_core::types::ResourceId;

#[test]
fn new_should_work() {
    let p = "/tmp/test";
    let path = PathBuf::from(p);
    let rid = ResourceId::new();
    let prj = Project::new(rid.clone(), path);

    assert_eq!(PathBuf::from(p), prj.path, "path should be set");
    assert_eq!(rid, prj.rid, "resource id should be set");
}
