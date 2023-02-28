use super::*;
use directories::ProjectDirs;
use thot_core::identifier::Identifier;

#[test]
fn user_settings_dir_path_should_work() {
    let ref_dirs = ProjectDirs::from(
        &Identifier::qualifier(),
        &Identifier::organization(),
        &Identifier::application(),
    )
    .expect("could not find project directories.");

    match UserSettings::dir_path() {
        Ok(prj_dirs) => {
            assert_eq!(ref_dirs.config_dir(), prj_dirs, "incorrect directory");
        }
        Err(err) => {
            assert_eq!(false, true, "should not error: {:?}", err);
        }
    }
}

#[test]
fn user_settings_file_path_should_work() {
    let ref_dirs = ProjectDirs::from(
        &Identifier::qualifier(),
        &Identifier::organization(),
        &Identifier::application(),
    )
    .expect("could not load user settings directory");

    let ref_path = ref_dirs.config_dir().join("settings.json");
    match UserSettings::path() {
        Ok(path) => {
            assert_eq!(
                ref_path, path,
                "should be in the settings directory with file name settings.json"
            );
        }
        Err(err) => {
            panic!("should not error: {:?}", err);
        }
    }
}

#[test]
fn user_settings_load_should_work() {
    let _settings = match UserSettings::load() {
        Ok(sets) => sets,
        Err(err) => {
            panic!("should not error: {:?}", err);
        }
    };
}

#[test]
fn user_settings_save_should_work() {
    let mut settings = UserSettings::default();
    settings.acquire_lock().expect("file lock not acquired");

    if let Err(err) = settings.save() {
        panic!("should not cause error: {:?}", err);
    };
}

#[test]
fn user_settings_save_should_error_if_lock_not_obtained() {
    let mut settings = UserSettings::default();
    assert!(
        settings.file().is_none(),
        "default settings should not control file initially"
    );

    match settings.save() {
        Ok(_) => {
            panic!("settings saved without having obtained lock");
        }
        Err(err) => {
            match err {
                SettingsError::IoError(ioerr)
                    if ioerr.kind() == io::ErrorKind::PermissionDenied =>
                {
                    // correct, pass
                }

                _ => {
                    panic!("unexpected error kind: {:?}", err);
                }
            }
        }
    };
}

#[test]
fn user_settings_default_should_not_acquire_file_lock() {
    let settings = UserSettings::default();
    assert!(
        !settings.file().is_none(),
        "default user settings should not lock file"
    )
}
