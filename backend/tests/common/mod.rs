use axum::Router;
use axum_test::{TestServer, TestServerConfig};
use backend::{
    email::{EmailClient, MockEmailClient},
    prelude::{AppState, Config},
    routes,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};
use tower_http::trace::TraceLayer;

/// A running integration-test environment.
///
/// Holds the testcontainers handle so the container stays alive for the
/// duration of the test, plus the [`TestServer`] and a shared
/// [`MockEmailClient`] for assertions.
pub struct TestApp {
    pub server: TestServer,
    pub mock_email: Arc<MockEmailClient>,
    pub db: PgPool,
    /// Keep the container alive for the lifetime of the test.
    _container: testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
}

impl TestApp {
    /// Spin up a Postgres container, run migrations, and return a ready
    /// [`TestApp`] backed by a [`MockEmailClient`].
    pub async fn spawn() -> Self {
        let container = Postgres::default().start().await.unwrap();
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();

        let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

        let db = PgPool::connect(&database_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();

        let config = test_config(database_url);
        let mock_email = Arc::new(MockEmailClient::new());

        let state = AppState::with_email(
            config,
            db.clone(),
            mock_email.clone() as Arc<dyn EmailClient>,
        );

        let app = Router::new()
            .nest("/api", routes::routes())
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let test_config = TestServerConfig {
            save_cookies: true,
            default_content_type: Some("application/json".to_string()),
            expect_success_by_default: true,
            ..TestServerConfig::default()
        };

        let server = TestServer::new_with_config(app, test_config).unwrap();

        Self {
            server,
            mock_email,
            db,
            _container: container,
        }
    }

    /// Register a regular user and return the response body.
    /// The session cookie is automatically saved by `axum_test`.
    pub async fn register(&self, username: &str, email: &str) -> serde_json::Value {
        let response = self
            .server
            .post("/api/users/register")
            .json(&json!({
                "username": username,
                "email": email,
                "password": "Str0ng!Pass"
            }))
            .await;
        response.json()
    }

    /// Register a user, then promote them to admin via a direct DB update.
    /// Re-authenticates so the JWT contains `admin: true`.
    pub async fn register_admin(&self, username: &str, email: &str) -> serde_json::Value {
        self.register(username, email).await;

        // Promote to admin in the database
        sqlx::query("UPDATE users SET admin = TRUE WHERE username = $1")
            .bind(username)
            .execute(&self.db)
            .await
            .unwrap();

        // Re-authenticate to get a JWT with admin=true
        let response = self
            .server
            .post("/api/users/auth")
            .json(&json!({
                "username": username,
                "password": "Str0ng!Pass"
            }))
            .await;
        response.json()
    }
}

/// Build a [`Config`] suitable for tests.
///
/// Mailjet credentials are dummies — the [`MockEmailClient`] is used instead.
fn test_config(database_url: String) -> Config {
    Config {
        port: 0,
        jwt_secret: "test-jwt-secret-that-is-long-enough".to_string(),
        database_url,
        mailjet_api_key: "test-key".to_string(),
        mailjet_api_secret: "test-secret".to_string(),
        url: "http://localhost:3000".to_string(),
        mail_from_email: "noreply@test.local".to_string(),
        mail_from_name: "Test".to_string(),
    }
}
