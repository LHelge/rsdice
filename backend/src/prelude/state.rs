use crate::email::{EmailClient, MailjetClient};
use crate::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub email: Arc<dyn EmailClient>,
}

impl AppState {
    pub fn new(config: Config, db: PgPool) -> Self {
        Self {
            email: Arc::new(MailjetClient::new(&config)),
            config: Arc::new(config),
            db,
        }
    }
}
