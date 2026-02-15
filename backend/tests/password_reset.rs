mod common;

use backend::email::MailType;
use common::TestApp;
use serde_json::json;
use sha2::{Digest, Sha256};

// ==== Request Password Reset ====

#[tokio::test]
async fn request_password_reset_by_username_sends_email() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice" }))
        .await;

    let sent = app.mock_email.sent();
    assert_eq!(sent.len(), 2);
    assert!(matches!(sent[1].mail_type, MailType::PasswordReset { .. }));
}

#[tokio::test]
async fn request_password_reset_by_email_sends_email() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice@example.com" }))
        .await;

    let sent = app.mock_email.sent();
    assert_eq!(sent.len(), 2);
    assert!(matches!(sent[1].mail_type, MailType::PasswordReset { .. }));
}

#[tokio::test]
async fn request_password_reset_for_unknown_account_returns_success_without_email() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "does-not-exist" }))
        .await;

    response.assert_status_ok();
    assert_eq!(app.mock_email.sent().len(), 0);
}

// ==== Confirm Password Reset ====

#[tokio::test]
async fn reset_password_with_valid_token_updates_password_and_revokes_sessions() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice" }))
        .await;

    let token = match app.mock_email.latest().unwrap().mail_type {
        MailType::PasswordReset { token } => token,
        _ => panic!("expected PasswordReset"),
    };

    app.server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": token,
            "password": "N3w!Passw0rd"
        }))
        .await;

    let refresh_response = app.server.post("/api/users/refresh").expect_failure().await;
    refresh_response.assert_status_unauthorized();

    let old_login = app
        .server
        .post("/api/users/auth")
        .json(&json!({
            "username": "alice",
            "password": "Str0ng!Pass"
        }))
        .expect_failure()
        .await;
    old_login.assert_status_bad_request();

    app.server
        .post("/api/users/auth")
        .json(&json!({
            "username": "alice",
            "password": "N3w!Passw0rd"
        }))
        .await;
}

#[tokio::test]
async fn reset_password_token_cannot_be_reused() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice" }))
        .await;

    let token = match app.mock_email.latest().unwrap().mail_type {
        MailType::PasswordReset { token } => token,
        _ => panic!("expected PasswordReset"),
    };

    app.server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": token,
            "password": "N3w!Passw0rd"
        }))
        .await;

    let second_attempt = app
        .server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": token,
            "password": "An0ther!Pass"
        }))
        .expect_failure()
        .await;

    second_attempt.assert_status_bad_request();
}

#[tokio::test]
async fn reset_password_with_older_token_fails_after_new_request() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice" }))
        .await;

    let first_token = match app.mock_email.latest().unwrap().mail_type {
        MailType::PasswordReset { token } => token,
        _ => panic!("expected PasswordReset"),
    };

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice" }))
        .await;

    let second_token = match app.mock_email.latest().unwrap().mail_type {
        MailType::PasswordReset { token } => token,
        _ => panic!("expected PasswordReset"),
    };

    let old_token_response = app
        .server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": first_token,
            "password": "N3w!Passw0rd"
        }))
        .expect_failure()
        .await;
    old_token_response.assert_status_bad_request();

    app.server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": second_token,
            "password": "N3w!Passw0rd"
        }))
        .await;
}

#[tokio::test]
async fn reset_password_with_invalid_token_fails() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": "bogus-token-value",
            "password": "N3w!Passw0rd"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn reset_password_with_expired_token_fails() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice" }))
        .await;

    let token = match app.mock_email.latest().unwrap().mail_type {
        MailType::PasswordReset { token } => token,
        _ => panic!("expected PasswordReset"),
    };

    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    sqlx::query(
        r#"
        UPDATE password_reset_tokens
        SET expires_at = NOW() - INTERVAL '1 minute'
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .execute(&app.db)
    .await
    .unwrap();

    let response = app
        .server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": token,
            "password": "N3w!Passw0rd"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn reset_password_with_weak_password_fails() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    app.server
        .post("/api/users/request-password-reset")
        .json(&json!({ "identifier": "alice" }))
        .await;

    let token = match app.mock_email.latest().unwrap().mail_type {
        MailType::PasswordReset { token } => token,
        _ => panic!("expected PasswordReset"),
    };

    let response = app
        .server
        .post("/api/users/reset-password")
        .json(&json!({
            "token": token,
            "password": "weak"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}
