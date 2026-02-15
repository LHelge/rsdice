use super::{EmailClient, EmailError, Mail};
use crate::prelude::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};
use tracing::{debug, error};
use uuid::Uuid;

// ============================================================================
// Mailjet API wire types (private to this module)
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct EmailAddress {
    email: String,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Message {
    from: EmailAddress,
    to: Vec<EmailAddress>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<EmailAddress>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<EmailAddress>,
    subject: String,
    text_part: String,
    html_part: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Messages {
    messages: Vec<Message>,
}

// ============================================================================
// Mailjet API response types
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SendStatus {
    Success,
    Error,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EmailStatus {
    email: String,
    #[serde(rename = "MessageUUID")]
    message_uuid: Uuid,
    #[serde(rename = "MessageID")]
    message_id: u64,
    message_href: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ErrorStatus {
    error_identifier: Uuid,
    error_code: String,
    status_code: u16,
    error_message: String,
    error_related_to: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MessageStatus {
    status: SendStatus,
    #[serde(default)]
    to: Vec<EmailStatus>,
    #[serde(default)]
    cc: Vec<EmailStatus>,
    #[serde(default)]
    bcc: Vec<EmailStatus>,
    #[serde(default)]
    errors: Vec<ErrorStatus>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MessageResponse {
    messages: Vec<MessageStatus>,
}

// ============================================================================
// MailjetClient
// ============================================================================

/// Mailjet-backed [`EmailClient`] implementation.
///
/// Sends transactional email via the Mailjet v3.1 Send API. Credentials and
/// sender identity are stored at construction time. The struct is cheap to
/// clone thanks to internal `Arc`s and a shared `reqwest::Client`.
#[derive(Debug)]
pub struct MailjetClient {
    api_key: String,
    api_secret: String,
    from_email: String,
    from_name: String,
    base_url: String,
    client: Client,
}

impl MailjetClient {
    const MAILJET_API_URL: &'static str = "https://api.mailjet.com/v3.1";

    /// Create a new [`MailjetClient`] from application configuration.
    pub fn new(config: &Config) -> Self {
        Self {
            api_key: config.mailjet_api_key.clone(),
            api_secret: config.mailjet_api_secret.clone(),
            from_email: config.mail_from_email.clone(),
            from_name: config.mail_from_name.clone(),
            base_url: config.url.clone(),
            client: Client::new(),
        }
    }
}

impl EmailClient for MailjetClient {
    fn send<'a>(
        &'a self,
        mail: &'a Mail,
    ) -> Pin<Box<dyn Future<Output = Result<(), EmailError>> + Send + 'a>> {
        Box::pin(async move {
            let recipient = &mail.recipient;

            let html_part = mail.to_html(&self.base_url)?;
            let text_part = mail.to_text(&self.base_url);

            let messages = Messages {
                messages: vec![Message {
                    from: EmailAddress {
                        email: self.from_email.clone(),
                        name: self.from_name.clone(),
                    },
                    to: vec![EmailAddress {
                        email: recipient.email.clone(),
                        name: recipient.name.clone(),
                    }],
                    cc: vec![],
                    bcc: vec![],
                    subject: mail.subject().to_string(),
                    text_part,
                    html_part,
                }],
            };

            let message = serde_json::to_string(&messages)
                .unwrap_or("Failed to serialize message".to_string());
            debug!("Email payload: {}", message);

            let url = format!("{}/send", Self::MAILJET_API_URL);
            debug!("Mailjet API URL: {}", url);

            let response = self
                .client
                .post(&url)
                .basic_auth(&self.api_key, Some(&self.api_secret))
                .json(&messages)
                .send()
                .await?;

            if response.status().is_success() {
                let message_response: MessageResponse = response.json().await?;
                debug!("Mailjet response: {:?}", message_response);
            } else {
                let error_response: MessageResponse = response.json().await?;
                error!("Mailjet error response: {:?}", error_response);
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==== Serialization Tests ====

    #[test]
    fn serializes_basic_send_api_v31_payload() {
        let payload = Messages {
            messages: vec![Message {
                from: EmailAddress {
                    email: "pilot@mailjet.com".to_string(),
                    name: "Mailjet Pilot".to_string(),
                },
                to: vec![EmailAddress {
                    email: "passenger1@mailjet.com".to_string(),
                    name: "passenger 1".to_string(),
                }],
                cc: vec![],
                bcc: vec![],
                subject: "Your email flight plan!".to_string(),
                text_part:
                    "Dear passenger 1, welcome to Mailjet! May the delivery force be with you!"
                        .to_string(),
                html_part: "<h3>Dear passenger 1, welcome to <a href=\"https://www.mailjet.com/\">Mailjet</a>!</h3><br />May the delivery force be with you!".to_string(),
            }],
        };

        let value = serde_json::to_value(&payload).unwrap();
        let message = &value["Messages"][0];

        assert_eq!(message["From"]["Email"], "pilot@mailjet.com");
        assert_eq!(message["From"]["Name"], "Mailjet Pilot");
        assert_eq!(message["To"][0]["Email"], "passenger1@mailjet.com");
        assert_eq!(message["To"][0]["Name"], "passenger 1");
        assert_eq!(message["Subject"], "Your email flight plan!");
        assert!(message.get("Cc").is_none());
        assert!(message.get("Bcc").is_none());
    }

    // ==== Deserialization Tests ====

    #[test]
    fn deserializes_basic_success_response() {
        let json = r#"
        {
            "Messages": [
                {
                    "Status": "success",
                    "To": [
                        {
                            "Email": "passenger1@mailjet.com",
                            "MessageUUID": "00000000-0000-0000-0000-000000000001",
                            "MessageID": 456,
                            "MessageHref": "https://api.mailjet.com/v3/message/456"
                        }
                    ]
                }
            ]
        }
        "#;

        let response: MessageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.messages.len(), 1);
        assert!(matches!(response.messages[0].status, SendStatus::Success));
        assert_eq!(response.messages[0].to.len(), 1);
        assert_eq!(response.messages[0].to[0].email, "passenger1@mailjet.com");
        assert_eq!(response.messages[0].to[0].message_id, 456);
    }

    #[test]
    fn deserializes_error_response_without_to_cc_bcc() {
        let json = r#"
        {
            "Messages": [
                {
                    "Status": "error",
                    "Errors": [
                        {
                            "ErrorIdentifier": "00000000-0000-0000-0000-000000000002",
                            "ErrorCode": "send-0003",
                            "StatusCode": 400,
                            "ErrorMessage": "At least \"HTMLPart\", \"TextPart\" or \"TemplateID\" must be provided.",
                            "ErrorRelatedTo": ["HTMLPart", "TextPart"]
                        }
                    ]
                }
            ]
        }
        "#;

        let response: MessageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.messages.len(), 1);
        assert!(matches!(response.messages[0].status, SendStatus::Error));
        assert!(response.messages[0].to.is_empty());
        assert!(response.messages[0].cc.is_empty());
        assert!(response.messages[0].bcc.is_empty());
        assert_eq!(response.messages[0].errors.len(), 1);
        assert_eq!(response.messages[0].errors[0].error_code, "send-0003");
    }
}
