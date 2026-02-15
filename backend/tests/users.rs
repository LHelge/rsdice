mod common;

use common::TestApp;
use serde_json::json;

// ==== List Users (admin) ====

#[tokio::test]
async fn list_users_as_admin_succeeds() {
    let app = TestApp::spawn().await;
    app.register_admin("admin", "admin@example.com").await;

    let response = app.server.get("/api/users").await;
    response.assert_status_ok();

    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["username"], "admin");
}

#[tokio::test]
async fn list_users_as_non_admin_fails() {
    let app = TestApp::spawn().await;
    app.register("alice", "alice@example.com").await;

    let response = app.server.get("/api/users").expect_failure().await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn list_users_unauthenticated_fails() {
    let app = TestApp::spawn().await;

    let response = app.server.get("/api/users").expect_failure().await;
    response.assert_status_unauthorized();
}

// ==== Create User (admin) ====

#[tokio::test]
async fn create_user_as_admin_succeeds() {
    let app = TestApp::spawn().await;
    app.register_admin("admin", "admin@example.com").await;

    let response = app
        .server
        .post("/api/users")
        .json(&json!({
            "username": "newuser",
            "email": "new@example.com",
            "password": "Str0ng!Pass",
            "admin": false
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "newuser");
    assert_eq!(body["admin"], false);
}

#[tokio::test]
async fn create_user_as_non_admin_fails() {
    let app = TestApp::spawn().await;
    app.register("alice", "alice@example.com").await;

    let response = app
        .server
        .post("/api/users")
        .json(&json!({
            "username": "newuser",
            "email": "new@example.com",
            "password": "Str0ng!Pass",
            "admin": false
        }))
        .expect_failure()
        .await;

    response.assert_status_not_found();
}

// ==== Get User ====

#[tokio::test]
async fn get_own_user_succeeds() {
    let app = TestApp::spawn().await;
    let user: serde_json::Value = app.register("alice", "alice@example.com").await;
    let id = user["id"].as_str().unwrap();

    let response = app.server.get(&format!("/api/users/{id}")).await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "alice");
}

#[tokio::test]
async fn get_other_user_as_non_admin_fails() {
    let app = TestApp::spawn().await;

    // Register alice (cookie saved)
    app.register("alice", "alice@example.com").await;

    // Create bob directly in the DB so we have another user ID
    let bob_uuid = uuid::Uuid::new_v4();
    let bob_id: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (id, username, email, password_hash, admin) VALUES ($1, 'bob', 'bob@example.com', 'hash', false) RETURNING id",
    )
    .bind(bob_uuid)
    .fetch_one(&app.db)
    .await
    .unwrap();

    let response = app
        .server
        .get(&format!("/api/users/{}", bob_id.0))
        .expect_failure()
        .await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn get_any_user_as_admin_succeeds() {
    let app = TestApp::spawn().await;
    app.register_admin("admin", "admin@example.com").await;

    // Create bob via the admin API
    let bob: serde_json::Value = app
        .server
        .post("/api/users")
        .json(&json!({
            "username": "bob",
            "email": "bob@example.com",
            "password": "Str0ng!Pass",
            "admin": false
        }))
        .await
        .json();
    let bob_id = bob["id"].as_str().unwrap();

    let response = app.server.get(&format!("/api/users/{bob_id}")).await;
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "bob");
}

// ==== Update User (admin) ====

#[tokio::test]
async fn update_user_as_admin_succeeds() {
    let app = TestApp::spawn().await;
    app.register_admin("admin", "admin@example.com").await;

    let bob: serde_json::Value = app
        .server
        .post("/api/users")
        .json(&json!({
            "username": "bob",
            "email": "bob@example.com",
            "password": "Str0ng!Pass",
            "admin": false
        }))
        .await
        .json();
    let bob_id = bob["id"].as_str().unwrap();

    let response = app
        .server
        .put(&format!("/api/users/{bob_id}"))
        .json(&json!({
            "username": "bobby",
            "admin": true
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "bobby");
    assert_eq!(body["admin"], true);
}

#[tokio::test]
async fn update_user_as_non_admin_fails() {
    let app = TestApp::spawn().await;
    let user: serde_json::Value = app.register("alice", "alice@example.com").await;
    let id = user["id"].as_str().unwrap();

    let response = app
        .server
        .put(&format!("/api/users/{id}"))
        .json(&json!({
            "username": "newname",
            "admin": false
        }))
        .expect_failure()
        .await;

    response.assert_status_not_found();
}

// ==== Update Password ====

#[tokio::test]
async fn update_own_password_succeeds() {
    let app = TestApp::spawn().await;
    let user: serde_json::Value = app.register("alice", "alice@example.com").await;
    let id = user["id"].as_str().unwrap();

    app.server
        .post(&format!("/api/users/{id}/password"))
        .json(&json!({ "password": "NewStr0ng!Pass" }))
        .await;

    // Authenticate with the new password
    app.server
        .post("/api/users/auth")
        .json(&json!({
            "username": "alice",
            "password": "NewStr0ng!Pass"
        }))
        .await;
}

#[tokio::test]
async fn update_other_users_password_as_non_admin_fails() {
    let app = TestApp::spawn().await;
    app.register("alice", "alice@example.com").await;

    let bob_uuid = uuid::Uuid::new_v4();
    let bob_id: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (id, username, email, password_hash, admin) VALUES ($1, 'bob', 'bob@example.com', 'hash', false) RETURNING id",
    )
    .bind(bob_uuid)
    .fetch_one(&app.db)
    .await
    .unwrap();

    let response = app
        .server
        .post(&format!("/api/users/{}/password", bob_id.0))
        .json(&json!({ "password": "NewStr0ng!Pass" }))
        .expect_failure()
        .await;

    response.assert_status_not_found();
}

// ==== Delete User (admin) ====

#[tokio::test]
async fn delete_user_as_admin_succeeds() {
    let app = TestApp::spawn().await;
    app.register_admin("admin", "admin@example.com").await;

    let bob: serde_json::Value = app
        .server
        .post("/api/users")
        .json(&json!({
            "username": "bob",
            "email": "bob@example.com",
            "password": "Str0ng!Pass",
            "admin": false
        }))
        .await
        .json();
    let bob_id = bob["id"].as_str().unwrap();

    app.server.delete(&format!("/api/users/{bob_id}")).await;

    // Verify bob no longer exists
    let response = app
        .server
        .get(&format!("/api/users/{bob_id}"))
        .expect_failure()
        .await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn delete_user_as_non_admin_fails() {
    let app = TestApp::spawn().await;
    let user: serde_json::Value = app.register("alice", "alice@example.com").await;
    let id = user["id"].as_str().unwrap();

    let response = app
        .server
        .delete(&format!("/api/users/{id}"))
        .expect_failure()
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn delete_nonexistent_user_returns_not_found() {
    let app = TestApp::spawn().await;
    app.register_admin("admin", "admin@example.com").await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .server
        .delete(&format!("/api/users/{fake_id}"))
        .expect_failure()
        .await;

    response.assert_status_not_found();
}
