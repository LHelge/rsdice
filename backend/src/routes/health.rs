use crate::prelude::*;
use axum::{Router, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(health))
}

async fn health() -> &'static str {
    "OK"
}
