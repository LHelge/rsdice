mod common;

use common::TestApp;
use serde_json::json;
use uuid::Uuid;

// Realtime test intent:
// - Contract checks verify endpoint shape/protocol guarantees (status/content-type/message schema).
// - Flow checks verify end-to-end behavior across actions (create, connect, command, emitted events).
// Keep contract assertions stable; extend flow assertions as gameplay protocol evolves.

// ==== SSE game list stream ====

#[tokio::test]
async fn games_stream_returns_event_stream_content_type() {
    let app = TestApp::spawn_http().await;
    let response = app.server.get("/api/games/stream").await;

    response.assert_status_ok();
    let content_type_header = response.header("content-type");
    let content_type = content_type_header.to_str().unwrap();
    assert!(
        content_type.starts_with("text/event-stream"),
        "unexpected content-type: {content_type}"
    );
}

#[tokio::test]
async fn games_stream_includes_games_event_payload() {
    let app = TestApp::spawn_http().await;

    app.register("alice", "alice@example.com").await;
    let created: serde_json::Value = app.server.put("/api/games").await.json();
    let game_id = created["id"].as_str().unwrap();

    let response = app.server.get("/api/games/stream").await;
    response.assert_status_ok();

    let body = response.text();
    assert!(body.contains("event: games"), "unexpected SSE body: {body}");
    assert!(body.contains(game_id), "unexpected SSE body: {body}");
    assert!(
        body.contains("\"player_count\":0"),
        "unexpected SSE body: {body}"
    );
}

#[tokio::test]
async fn games_stream_emits_empty_list_when_no_games_exist() {
    let app = TestApp::spawn_http().await;

    let response = app.server.get("/api/games/stream").await;
    response.assert_status_ok();

    let body = response.text();
    assert!(body.contains("event: games"), "unexpected SSE body: {body}");
    assert!(body.contains("data: []"), "unexpected SSE body: {body}");
}

// ==== Active game websocket ====

#[tokio::test]
async fn game_websocket_requires_authentication() {
    let app = TestApp::spawn_http().await;

    let response = app
        .server
        .get_websocket(&format!("/api/games/{}/ws", Uuid::new_v4()))
        .expect_failure()
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn game_websocket_returns_not_found_for_missing_game() {
    let app = TestApp::spawn_http().await;
    app.register("alice", "alice@example.com").await;

    let response = app
        .server
        .get_websocket(&format!("/api/games/{}/ws", Uuid::new_v4()))
        .expect_failure()
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn game_websocket_allows_commands_and_emits_events() {
    let app = TestApp::spawn_http().await;

    app.register("alice", "alice@example.com").await;
    let created: serde_json::Value = app.server.put("/api/games").await.json();
    let game_id = created["id"].as_str().unwrap();

    let mut alice_ws = app
        .server
        .get_websocket(&format!("/api/games/{game_id}/ws"))
        .await
        .into_websocket()
        .await;

    let alice_initial = alice_ws.receive_json::<serde_json::Value>().await;
    assert_eq!(alice_initial["type"], "snapshot");
    assert_eq!(alice_initial["game"]["id"], game_id);

    app.register("bob", "bob@example.com").await;
    let mut bob_ws = app
        .server
        .get_websocket(&format!("/api/games/{game_id}/ws"))
        .await
        .into_websocket()
        .await;

    let bob_initial = bob_ws.receive_json::<serde_json::Value>().await;
    assert_eq!(bob_initial["type"], "snapshot");

    bob_ws.send_json(&json!({ "type": "start" })).await;

    let mut saw_game_started = false;
    for _ in 0..4 {
        let event = bob_ws.receive_json::<serde_json::Value>().await;
        if event["type"] == "game_started" {
            saw_game_started = true;
            break;
        }
    }

    assert!(saw_game_started, "expected game_started event on websocket");

    bob_ws.send_text("not-json").await;
    let error_event = bob_ws.receive_json::<serde_json::Value>().await;
    assert_eq!(error_event["type"], "error");
    assert!(
        error_event["message"]
            .as_str()
            .unwrap()
            .contains("invalid command payload")
    );
}

#[tokio::test]
async fn game_websocket_reconnect_same_user_receives_snapshot() {
    let app = TestApp::spawn_http().await;

    app.register("alice", "alice@example.com").await;
    let created: serde_json::Value = app.server.put("/api/games").await.json();
    let game_id = created["id"].as_str().unwrap();

    let mut first_ws = app
        .server
        .get_websocket(&format!("/api/games/{game_id}/ws"))
        .await
        .into_websocket()
        .await;

    let first_snapshot = first_ws.receive_json::<serde_json::Value>().await;
    assert_eq!(first_snapshot["type"], "snapshot");
    assert_eq!(first_snapshot["game"]["id"], game_id);

    let mut second_ws = app
        .server
        .get_websocket(&format!("/api/games/{game_id}/ws"))
        .await
        .into_websocket()
        .await;

    let second_snapshot = second_ws.receive_json::<serde_json::Value>().await;
    assert_eq!(second_snapshot["type"], "snapshot");
    assert_eq!(second_snapshot["game"]["id"], game_id);
}
