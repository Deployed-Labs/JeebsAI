use jeebs::auth::AuthStatusResponse;

#[test]
fn trainer_flag_parses_from_auth_status() {
    let payload = serde_json::json!({
        "logged_in": true,
        "username": "trainer_user",
        "is_admin": false,
        "is_trainer": true,
        "token": null
    });

    let parsed: AuthStatusResponse = serde_json::from_value(payload).expect("parse auth status");
    assert!(parsed.logged_in);
    assert_eq!(parsed.username.as_deref(), Some("trainer_user"));
    assert!(!parsed.is_admin);
    assert!(parsed.is_trainer);
}
