use super::*;
use std::io;
use settings_manager::result::Error as SettingsError;
use thot_core::result::Error as ThotError;
use clap::Command;
use clap::error::ErrorKind as ClapErrorKind;

#[test]
fn error_from_clap_error_should_work() {
    let cmd = Command::new("test");
    let o_err = cmd.error(ClapErrorKind::InvalidValue, "test");
    
    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::ClapError(_)));
}

#[test]
fn error_from_io_error_should_work() {
    let o_err = io::Error::new(
        io::ErrorKind::Other,
        "test"
    );

    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::IoError(_)));
}

#[test]
fn error_from_settings_manager_error_should_work() {
    let io_err = io::Error::new(
        io::ErrorKind::Other,
        "test"
    );

    let o_err = SettingsError::IoError(io_err);
    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::SettingsError(_)));
}

#[test]
fn error_from_thot_core_error_should_work() {
    let io_err = io::Error::new(
        io::ErrorKind::Other,
        "test"
    );

    let o_err = ThotError::IoError(io_err);
    let c_err: Error = o_err.into();
    assert!(matches!(c_err, Error::ThotError(_)));
}
