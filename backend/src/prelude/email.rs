#![allow(dead_code)]

use super::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Represents an email participant used by Mailjet payloads.
///
/// Mailjet expects recipient and sender objects to use PascalCase keys,
/// therefore this type serializes as:
///
/// - `Email`
/// - `Name`
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EmailAddress {
    /// The email address in RFC-compatible string form.
    pub email: String,
    /// Human-readable display name associated with the address.
    pub name: String,
}

/// A single outbound email message submitted to Mailjet.
///
/// This struct mirrors one element of the `Messages` array in the Mailjet
/// v3.1 send API.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    /// Sender information.
    pub from: EmailAddress,
    /// Primary recipients.
    pub to: Vec<EmailAddress>,

    /// Carbon-copy recipients.
    ///
    /// Omitted from serialized JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<EmailAddress>,

    /// Blind carbon-copy recipients.
    ///
    /// Omitted from serialized JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<EmailAddress>,

    /// Message subject line.
    pub subject: String,
    /// Plain-text representation of the message body.
    pub text_part: String,
    /// HTML representation of the message body.
    pub html_part: String,
}

/// Top-level payload for Mailjet send operations.
///
/// Mailjet v3.1 accepts a `Messages` array that can contain one or many
/// individual messages. This wrapper models that request shape.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Messages {
    /// List of messages to send in a single API request.
    pub messages: Vec<Message>,
}

/// Delivery status returned by Mailjet for message or recipient processing.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SendStatus {
    /// Operation completed successfully.
    Success,
    /// Operation failed.
    Error,
}

/// Success metadata for an accepted recipient.
///
/// Mailjet returns this object for recipients where processing succeeded.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EmailStatus {
    /// Recipient email address associated with the result.
    pub email: String,
    /// Unique Mailjet UUID for the message attempt.
    pub message_uuid: Uuid,
    /// Numeric Mailjet message identifier.
    pub message_id: u64,
    /// API URL that references the message resource.
    pub message_href: String,
}

/// Error details reported by Mailjet for a failed message operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EmailError {
    /// Unique identifier for this error instance.
    pub error_identifier: Uuid,
    /// Provider-specific machine-readable error code.
    pub error_code: String,
    /// HTTP status code related to the failure.
    pub status_code: u16,
    /// Human-readable failure description.
    pub error_message: String,
    /// Field names or entities associated with the error.
    pub error_related_to: Vec<String>,
}

/// Per-message status summary returned by Mailjet.
///
/// The API may omit `cc`, `bcc`, or `errors` arrays when they are empty.
/// Those fields default to empty vectors to keep the model ergonomic and avoid
/// additional `Option<Vec<_>>` handling.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageStatus {
    /// Overall status for the message.
    pub status: SendStatus,
    /// Status entries for all primary recipients.
    pub to: Vec<SendStatus>,
    /// Status entries for carbon-copy recipients.
    #[serde(default)]
    pub cc: Vec<SendStatus>,
    /// Status entries for blind carbon-copy recipients.
    #[serde(default)]
    pub bcc: Vec<SendStatus>,
    /// Error entries associated with this message.
    #[serde(default)]
    pub errors: Vec<EmailError>,
}

/// Minimal response model for a send call.
///
/// This currently captures only top-level aggregate status. Expand this type
/// if additional fields from the Mailjet response become relevant.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageResponse {
    /// Aggregate status for the submitted `messages` collection.
    pub messages: SendStatus,
}

/// Thin Mailjet API client used by backend services to send transactional email.
///
/// The client is cheap to clone and internally shares credentials through `Arc`.
/// It uses a reusable `reqwest::Client` for connection pooling.
#[derive(Debug, Clone)]
pub struct EmailClient {
    api_key: Arc<String>,
    api_secret: Arc<String>,
    client: Client,
}

impl EmailClient {
    /// Base URL for Mailjet API v3.1.
    const MAILJET_API_URL: &'static str = "https://api.mailjet.com/v3.1";

    /// Creates a new [`EmailClient`] from application configuration.
    ///
    /// The API credentials are cloned from `Config` and stored in shared
    /// pointers so cloned clients remain lightweight.
    pub fn new(config: &Config) -> Self {
        Self {
            api_key: Arc::new(config.mailjet_api_key.clone()),
            api_secret: Arc::new(config.mailjet_api_secret.clone()),
            client: Client::new(),
        }
    }

    /// Sends one or more messages using Mailjet's `/send` endpoint.
    ///
    /// The request is authenticated via HTTP Basic Auth using the configured
    /// API key and secret. Non-success HTTP responses are converted into
    /// `reqwest` errors via [`reqwest::Response::error_for_status`].
    ///
    /// # Errors
    ///
    /// Returns an error when network communication fails, Mailjet returns a
    /// non-2xx status, or the response body cannot be deserialized.
    pub async fn send_email(&self, messages: &Messages) -> reqwest::Result<MessageResponse> {
        self.client
            .post(format!("{}/send", Self::MAILJET_API_URL))
            .basic_auth(&*self.api_key, Some(&*self.api_secret))
            .json(&messages)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
