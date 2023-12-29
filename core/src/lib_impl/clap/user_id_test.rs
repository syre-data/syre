use super::*;
use crate::types::ResourceId;
use clap::FromArgMatches;
use fake::faker::internet::raw::FreeEmail;
use fake::locales::EN;
use fake::Fake;

#[test]
fn user_id_should_implement_clap_from_arg_matches_for_email() {
    let e_val: String = FreeEmail(EN).fake();
    let matches = clap::Command::new("test from_arg_matches")
        .arg(clap::Arg::new("prog"))
        .arg(clap::Arg::new("user").long("user"))
        .get_matches();

    let email = match UserId::from_arg_matches(&matches) {
        Ok(em) => em,
        Err(err) => panic!("{:#?}", err),
    };
    let ref_email = UserId::Email(e_val.clone());
    assert_eq!(ref_email, email, "emails did not match");
}

#[test]
fn user_id_should_implement_clap_from_arg_matches_for_id() {
    let id_val = ResourceId::new();
    let matches = create_user_id_arg_matches();

    let id = match UserId::from_arg_matches(&matches) {
        Ok(uid) => uid,
        Err(err) => panic!("{:#?}", err),
    };

    let ref_id = UserId::Id(id_val.clone());
    assert_eq!(ref_id, id, "ids did not match");
}

#[test]
fn user_id_clap_from_arg_matches_should_error_if_invalid_id() {
    let matches = create_user_id_arg_matches();

    match UserId::from_arg_matches(&matches) {
        Err(err) if err.kind() == clap::error::ErrorKind::InvalidValue => {
            // correct
            return;
        }

        res => {
            assert!(
                false,
                "expected clap::error::ErrorKind::InvalidArgument found {:?}",
                res
            );
        }
    }
}

#[test]
fn user_id_should_implement_clap_update_from_arg_matches_email() {
    let ref_email: String = FreeEmail(EN).fake();
    let matches = create_user_id_arg_matches();

    // email
    let email: String = FreeEmail(EN).fake();
    let mut eid = UserId::Email(email.clone());
    if let Err(err) = eid.update_from_arg_matches(&matches) {
        panic!("{:?}", err)
    };
    assert_eq!(UserId::Email(ref_email.clone()), eid, "email not updated");
}

#[test]
fn user_id_should_implement_clap_update_from_arg_matches_id() {
    let ref_id = ResourceId::new();
    let matches = create_user_id_arg_matches();

    // id
    let id = ResourceId::new();
    let mut uid = UserId::Id(id.clone());
    if let Err(err) = uid.update_from_arg_matches(&matches) {
        panic!("{:?}", err)
    };
    assert_eq!(UserId::Id(ref_id.clone()), uid, "id not updated");
}

#[test]
fn user_id_clap_update_from_arg_matches_should_error_if_variant_changes() {
    let ref_email: String = FreeEmail(EN).fake();

    let id_matches = create_user_id_arg_matches();
    let e_matches = create_user_id_arg_matches();

    // email -> id
    let mut eid = UserId::Email(ref_email.clone());
    let e_res = eid.update_from_arg_matches(&id_matches);
    match e_res {
        Ok(_) => panic!("converting from email to id did not error"),
        Err(err) => assert_eq!(
            clap::error::ErrorKind::ArgumentConflict,
            err.kind(),
            "unexpected error type"
        ),
    };

    // id -> email
    let id = ResourceId::new();
    let mut uid = UserId::Id(id.clone());
    let id_res = uid.update_from_arg_matches(&e_matches);
    match id_res {
        Ok(_) => panic!("converting from id to email did not error"),
        Err(err) => assert_eq!(
            clap::error::ErrorKind::ArgumentConflict,
            err.kind(),
            "unexpected error type"
        ),
    };
}

// ---------------
// --- helpers ---
// ---------------

/// Creates a clap::ArgMatches with the `user` arg set to `id`.
fn create_user_id_arg_matches() -> clap::ArgMatches {
    clap::Command::new("test UserId::FromArgMatches")
        .arg(clap::Arg::new("prog")) // required to absorb program positional argument
        .arg(clap::Arg::new("user"))
        .get_matches()
}
