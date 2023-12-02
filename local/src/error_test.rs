use super::*;
use std::io;
use thot_core::Error as CoreError;

#[test]
fn error_from_core_error_should_work() {
    let o_err = CoreError::IoError(io::Error::new(io::ErrorKind::Other, "test"));

    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::CoreError(_)));
}
