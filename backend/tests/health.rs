mod common;

use common::TestApp;

// ==== Health Check ====

#[tokio::test]
async fn health_returns_ok() {
    let app = TestApp::spawn().await;

    let response = app.server.get("/api/health").await;

    response.assert_status_ok();
    response.assert_text("OK");
}
