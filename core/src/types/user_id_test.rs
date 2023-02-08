use super::*;
use fake::faker::internet::raw::FreeEmail;
use fake::locales::EN;
use fake::Fake;
use std::str::FromStr;

// **************
// *** UserId ***
// **************

#[test]
fn user_id_equality_should_work() {
    // email
    let email: String = FreeEmail(EN).fake();
    let e0 = UserId::Email(email.clone());

    // equal email
    let e1 = UserId::Email(email.clone());
    assert_eq!(e0, e1, "emails should be equal");

    // unequal email
    let e1 = UserId::Email(FreeEmail(EN).fake());
    assert_ne!(e0, e1, "emails should not be equal");

    // ResourceId
    let uid = ResourceId::new();
    let u0 = UserId::Id(uid.clone());

    // equal uid
    let u1 = UserId::Id(uid.clone());
    assert_eq!(u0, u1, "user ids should be equal");

    // unequal uid
    let u1 = UserId::Id(ResourceId::new());
    assert_ne!(u0, u1, "user ids should not be equal");

    // email - id
    assert_ne!(e0, u0, "user id and email should never be equal");
}

#[test]
fn from_str_should_work_for_id() {
    let uid = ResourceId::new();
    let id = UserId::from_str(&uid.to_string()).expect("parse should not error");

    assert_eq!(UserId::Id(uid), id, "`ResourceId` was not parsed correctly");
}

#[test]
fn from_str_should_work_for_email() {
    let email: String = FreeEmail(EN).fake();
    let id = UserId::from_str(&email).expect("parse should not error");

    assert_eq!(UserId::Email(email), id, "email was not parsed correctly");
}

#[test]
fn from_str_should_error_for_invalid_input() {
    match UserId::from_str("invalid") {
        Ok(id) => assert!(false, "id should error when parsing, found {:?}", id),
        Err(ParseError(_)) => {}
    }
}
