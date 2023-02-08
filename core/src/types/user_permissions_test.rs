use super::*;

#[test]
fn user_permissions_new_should_work() {
    let perms = UserPermissions::new();

    assert_eq!(false, perms.read, "read permission should be false");
    assert_eq!(false, perms.write, "write permission should be false");
    assert_eq!(false, perms.execute, "execute permission should be false");
}
