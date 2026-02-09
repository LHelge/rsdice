use super::ClaimsError;
use crate::models::UserError;
use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Claims error: {0}")]
    Claims(#[from] ClaimsError),

    #[error("User error: {0}")]
    User(#[from] UserError),

    #[error("Not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Claims(e) => e.into_response(),
            Error::User(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            Error::NotFound => StatusCode::NOT_FOUND.into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
        }
    }
}
