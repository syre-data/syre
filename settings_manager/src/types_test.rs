use super::*;

#[test]
fn level_priority_test() {
    let s = Priority::System;
    let u = Priority::User;
    let l = Priority::Local;

    assert!(s < u, "system priority should be less than user");
    assert!(u < l, "user priority should be less than local");
}
