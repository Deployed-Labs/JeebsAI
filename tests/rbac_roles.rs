use jeebs::models::Role;

#[test]
fn test_role_parsing_from_various_strings() {
    assert_eq!(Role::from_str("GUEST"), Role::GUEST);
    assert_eq!(Role::from_str("guest"), Role::GUEST);
    assert_eq!(Role::from_str("USER"), Role::USER);
    assert_eq!(Role::from_str("user"), Role::USER);
    assert_eq!(Role::from_str("TRAINER"), Role::TRAINER);
    assert_eq!(Role::from_str("trainer"), Role::TRAINER);
    assert_eq!(Role::from_str("SUPERADMIN"), Role::SUPERADMIN);
    assert_eq!(Role::from_str("super_admin"), Role::SUPERADMIN);
    assert_eq!(Role::from_str("admin"), Role::SUPERADMIN);
    assert_eq!(Role::from_str("ADMIN"), Role::SUPERADMIN);
    // Unknown defaults to USER
    assert_eq!(Role::from_str("random"), Role::USER);
    assert_eq!(Role::from_str(""), Role::USER);
}

#[test]
fn test_role_level_hierarchy() {
    assert_eq!(Role::GUEST.level(), 0);
    assert_eq!(Role::USER.level(), 1);
    assert_eq!(Role::TRAINER.level(), 2);
    assert_eq!(Role::SUPERADMIN.level(), 3);
}

#[test]
fn test_role_has_access_matrix() {
    // SUPERADMIN can access everything
    assert!(Role::SUPERADMIN.has_access(&Role::GUEST));
    assert!(Role::SUPERADMIN.has_access(&Role::USER));
    assert!(Role::SUPERADMIN.has_access(&Role::TRAINER));
    assert!(Role::SUPERADMIN.has_access(&Role::SUPERADMIN));

    // TRAINER can access TRAINER, USER, GUEST
    assert!(Role::TRAINER.has_access(&Role::GUEST));
    assert!(Role::TRAINER.has_access(&Role::USER));
    assert!(Role::TRAINER.has_access(&Role::TRAINER));
    assert!(!Role::TRAINER.has_access(&Role::SUPERADMIN));

    // USER can access USER, GUEST
    assert!(Role::USER.has_access(&Role::GUEST));
    assert!(Role::USER.has_access(&Role::USER));
    assert!(!Role::USER.has_access(&Role::TRAINER));
    assert!(!Role::USER.has_access(&Role::SUPERADMIN));

    // GUEST can only access GUEST
    assert!(Role::GUEST.has_access(&Role::GUEST));
    assert!(!Role::GUEST.has_access(&Role::USER));
    assert!(!Role::GUEST.has_access(&Role::TRAINER));
    assert!(!Role::GUEST.has_access(&Role::SUPERADMIN));
}

#[test]
fn test_role_display_roundtrip() {
    for role in &[Role::GUEST, Role::USER, Role::TRAINER, Role::SUPERADMIN] {
        let display = role.to_string();
        let parsed = Role::from_str(&display);
        assert_eq!(*role, parsed);
    }
}

#[test]
fn test_role_serde_roundtrip() {
    for role in &[Role::GUEST, Role::USER, Role::TRAINER, Role::SUPERADMIN] {
        let json = serde_json::to_string(role).expect("serialize");
        let parsed: Role = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*role, parsed);
    }
}
