mod common;

use common::TestApp;
use serde_json::json;

// ==== Registration ====

#[tokio::test]
async fn register_succeeds_and_sets_cookie() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "alice");
    assert_eq!(body["email"], "alice@example.com");
    assert_eq!(body["admin"], false);
    assert!(body.get("password_hash").is_none());

    // A verification email should have been sent
    assert_eq!(app.mock_email.sent().len(), 1);
}

#[tokio::test]
async fn register_duplicate_username_fails() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    let response = app
        .server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "other@example.com",
            "password": "Str0ng!Pass"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn register_duplicate_email_fails() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    let response = app
        .server
        .post("/api/users/register")
        .json(&json!({
            "username": "bob",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn register_weak_password_fails() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "short"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn register_short_username_fails() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/register")
        .json(&json!({
            "username": "ab",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

// ==== Authentication ====

#[tokio::test]
async fn authenticate_succeeds_with_correct_credentials() {
    let app = TestApp::spawn().await;

    // Register first
    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    // Authenticate
    let response = app
        .server
        .post("/api/users/auth")
        .json(&json!({
            "username": "alice",
            "password": "Str0ng!Pass"
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "alice");
}

#[tokio::test]
async fn authenticate_wrong_password_fails() {
    let app = TestApp::spawn().await;

    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    let response = app
        .server
        .post("/api/users/auth")
        .json(&json!({
            "username": "alice",
            "password": "WrongPassword1!"
        }))
        .expect_failure()
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn authenticate_unknown_user_fails() {
    let app = TestApp::spawn().await;

    let response = app
        .server
        .post("/api/users/auth")
        .json(&json!({
            "username": "nonexistent",
            "password": "Str0ng!Pass"
        }))
        .expect_failure()
        .await;

    response.assert_status_not_found();
}

// ==== Me ====

#[tokio::test]
async fn me_returns_current_user_after_login() {
    let app = TestApp::spawn().await;

    // Register (cookie is saved automatically by axum_test)
    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    let response = app.server.get("/api/users/me").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "alice");
}

#[tokio::test]
async fn me_without_auth_returns_unauthorized() {
    let app = TestApp::spawn().await;

    let response = app.server.get("/api/users/me").expect_failure().await;
    response.assert_status_unauthorized();
}

// ==== Logout ====

#[tokio::test]
async fn logout_clears_session() {
    let app = TestApp::spawn().await;

    // Register (sets cookie)
    app.server
        .post("/api/users/register")
        .json(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "password": "Str0ng!Pass"
        }))
        .await;

    // Verify we're logged in
    app.server.get("/api/users/me").await;

    // Logout
    app.server.post("/api/users/logout").await;

    // me should now fail
    let response = app.server.get("/api/users/me").expect_failure().await;
    response.assert_status_unauthorized();
}
