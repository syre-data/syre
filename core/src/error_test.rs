use super::*;
use std::io;

#[test]
fn error_from_io_error_should_work() {
    let o_err = io::Error::new(io::ErrorKind::Other, "test");

    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::IoError(_)));
}

#[test]
fn error_from_serde_error_should_work() {
    todo!("don't know how to create serde error");
    // let o_err = serde_json::Error;
    // let c_err: Error = o_err.into();
    // assert!(matches!(c_err, Error::SettingsError(_)));
}
