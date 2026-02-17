mod games;
mod health;
mod users;

use crate::prelude::*;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/health", health::routes())
        .nest("/users", users::routes())
        .nest("/games", games::routes())
}
