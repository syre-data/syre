use super::*;
use settings_manager::Error as SettingsError;
use std::io;
use thot_core::Error as CoreError;

#[test]
fn error_from_core_error_should_work() {
    let o_err = CoreError::IoError(io::Error::new(io::ErrorKind::Other, "test"));

    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::CoreError(_)));
}

#[test]
fn error_from_settings_manager_error_should_work() {
    let io_err = io::Error::new(io::ErrorKind::Other, "test");

    let o_err = SettingsError::IoError(io_err);
    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::SettingsError(_)));
}
