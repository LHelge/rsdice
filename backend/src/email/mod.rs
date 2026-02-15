mod mail;
mod mailjet;
mod mock;

pub use mail::*;
pub use mailjet::*;
pub use mock::*;

use std::{future::Future, pin::Pin};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Failed to serialize email message: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error during email send: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Template rendering error: {0}")]
    Template(#[from] askama::Error),
}

/// Trait abstracting email delivery.
///
/// Implementations are responsible for converting a [`Mail`] into a
/// provider-specific payload and transmitting it. The from-address,
/// base URL, and any credentials should be stored on the implementing
/// struct so callers only need to supply the [`Mail`] value.
pub trait EmailClient: Send + Sync + std::fmt::Debug {
    /// Send an email described by `mail`.
    fn send<'a>(
        &'a self,
        mail: &'a Mail,
    ) -> Pin<Box<dyn Future<Output = Result<(), EmailError>> + Send + 'a>>;
}
