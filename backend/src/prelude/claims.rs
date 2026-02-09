use crate::prelude::*;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ClaimsError {
    #[error("Token is missing")]
    TokenMissing,

    #[error("Token is invalid: {0}")]
    TokenInvalid(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for ClaimsError {
    fn into_response(self) -> axum::response::Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

pub type ClaimsResult<T> = std::result::Result<T, ClaimsError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    iat: usize,
    pub sub: Uuid,
    pub admin: bool,
    pub username: String,
}

impl Claims {
    pub fn new(
        user_id: Uuid,
        admin: bool,
        username: impl Into<String>,
        lifetime: Duration,
    ) -> Self {
        let iat = chrono::Utc::now();
        let exp = iat + lifetime;

        Self {
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
            sub: user_id,
            admin,
            username: username.into(),
        }
    }

    pub fn encode(&self, secret: &str) -> ClaimsResult<String> {
        Ok(jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_ref()),
        )?)
    }

    pub fn decode(token: &str, secret: &str) -> ClaimsResult<Self> {
        let token = jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token.claims)
    }
}

//#[async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = ClaimsError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> ClaimsResult<Self> {
        let cookies = CookieJar::from_headers(&parts.headers);

        if let Some(token) = cookies.get("token") {
            Ok(Claims::decode(token.value(), &state.config.jwt_secret)?)
        } else {
            Err(ClaimsError::TokenMissing)
        }
    }
}
