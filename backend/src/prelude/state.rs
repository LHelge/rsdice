use crate::email::{EmailClient, MailjetClient};
use crate::games::Games;
use crate::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub email: Arc<dyn EmailClient>,
    pub games: Games,
}

impl AppState {
    pub fn new(config: Config, db: PgPool) -> Self {
        Self {
            email: Arc::new(MailjetClient::new(&config)),
            config: Arc::new(config),
            db,
            games: Games::default(),
        }
    }

    /// Create an `AppState` with a custom [`EmailClient`] implementation.
    ///
    /// Useful in tests where a [`MockEmailClient`](crate::email::MockEmailClient)
    /// replaces the real mail provider.
    pub fn with_email(config: Config, db: PgPool, email: Arc<dyn EmailClient>) -> Self {
        Self {
            config: Arc::new(config),
            db,
            email,
            games: Games::default(),
        }
    }
}
