use super::*;
use uuid::Uuid;
use thot_core::system::users;
use fake::faker::internet::raw::FreeEmail;
use fake::locales::EN;
use fake::Fake;
use settings_manager::prelude::SystemSettings;
use thot_core::system::settings::user_settings::UserSettings;
use thot_core::system::resources::{user::User, user_id::UserId};

#[test]
fn set_user_should_work_with_email() {
    let email: String = FreeEmail(EN).fake();
    let e_id = UserId::Email(email.clone());
    let user = User::new(email.clone(), None);
    let uid = Into::<Uuid>::into(user.rid.clone());
    users::add_user(user);
    set_active_user(&e_id);

    let settings = UserSettings::load().expect("user settings should load");
    
    assert_eq!(
        Some(uid), settings.active_user,
        "user should be set as active"
    );
}

#[test]
fn set_user_should_work_with_id() {
    let email: String = FreeEmail(EN).fake();
    let e_id = UserId::Email(email.clone());
    let user = User::new(email.clone(), None);
    let uid = UserId::Id(user.rid.clone());
    let uuid = Into::<Uuid>::into(user.rid.clone());

    users::add_user(user);
    set_active_user(&uid);

    let settings = UserSettings::load().expect("user settings should load");
    
    assert_eq!(
        Some(uuid), settings.active_user,
        "user should be set as active"
    );
}

#[test]
#[should_panic(expected = "DoesNotExist")]
fn set_user_with_id_should_error_if_the_user_does_not_exist() {
    let email: String = FreeEmail(EN).fake();
    let user = User::new(email, None);
    let uid = UserId::Id(user.rid.clone());
    match set_active_user(&uid) {
        Err(err) => panic!("{:?}", err),
        _ => (),
    };
}


#[test]
#[should_panic(expected = "DoesNotExist")]
fn set_user_with_email_should_error_if_the_user_does_not_exist() {
    let email: String = FreeEmail(EN).fake();
    let e_id = UserId::Email(email.clone());
    let user = User::new(email.clone(), None);
    match set_active_user(&e_id) {
        Err(err) => panic!("{:?}", err),
        _ => (),
    };
}

#[test]
fn set_user_should_error_if_the_user_is_invalid() {
    todo!();
}
