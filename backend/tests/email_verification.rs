mod common;

use backend::email::MailType;
use common::TestApp;
use serde_json::json;

// ==== Email Verification ====

#[tokio::test]
async fn register_sends_verification_email() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    let sent = app.mock_email.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].recipient.email, "alice@example.com");
    assert!(matches!(
        sent[0].mail_type,
        MailType::EmailVerification { .. }
    ));
}

#[tokio::test]
async fn verify_email_with_valid_token_succeeds() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    // Extract the token from the mock email
    let mail = app.mock_email.latest().expect("should have sent an email");
    let token = match mail.mail_type {
        MailType::EmailVerification { token } => token,
        _ => panic!("expected EmailVerification"),
    };

    // Verify the email
    app.server
        .post("/api/users/verify-email")
        .json(&json!({ "token": token }))
        .await;

    // Check user is now verified
    let response = app.server.get("/api/users/me").await;
    let body: serde_json::Value = response.json();
    assert_eq!(body["email_verified"], true);
}

#[tokio::test]
async fn verify_email_with_invalid_token_fails() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/verify-email")
        .json(&json!({ "token": "bogus-token-value" }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn verify_email_token_cannot_be_reused() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    let mail = app.mock_email.latest().unwrap();
    let token = match mail.mail_type {
        MailType::EmailVerification { token } => token,
        _ => panic!("expected EmailVerification"),
    };

    // First use succeeds
    app.server
        .post("/api/users/verify-email")
        .json(&json!({ "token": token }))
        .await;

    // Second use fails
    let response = app
        .server
        .post("/api/users/verify-email")
        .json(&json!({ "token": token }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn resend_verification_sends_new_email() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    assert_eq!(app.mock_email.sent().len(), 1);

    // Resend
    app.server.post("/api/users/resend-verification").await;

    assert_eq!(app.mock_email.sent().len(), 2);
}

#[tokio::test]
async fn resend_verification_after_verified_fails() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    // Verify
    let mail = app.mock_email.latest().unwrap();
    let token = match mail.mail_type {
        MailType::EmailVerification { token } => token,
        _ => panic!("expected EmailVerification"),
    };
    app.server
        .post("/api/users/verify-email")
        .json(&json!({ "token": token }))
        .await;

    // Resend should fail
    let response = app
        .server
        .post("/api/users/resend-verification")
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn resend_verification_without_auth_fails() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/resend-verification")
        .expect_failure()
        .await;

    response.assert_status_unauthorized();
}
