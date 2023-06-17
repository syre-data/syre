use super::*;
use crate::runner::env::CONTAINER_ID_KEY;
use std::env;

#[test]
fn dev_mode_should_work() {
    // no container id set
    assert_eq!(true, dev_mode(), "dev mode should be true");

    // set container id
    env::set_var(CONTAINER_ID_KEY, "TEST CONTAINER");
    assert_eq!(false, dev_mode(), "dev mode should be false");
}
