use super::{EmailClient, EmailError, Mail};
use std::{future::Future, pin::Pin, sync::Mutex};

/// Mock [`EmailClient`] that records every [`Mail`] sent.
///
/// Useful in integration and unit tests to assert that the correct emails
/// were dispatched without hitting a real mail provider.
///
/// # Example
///
/// ```rust,ignore
/// let mock = MockEmailClient::new();
/// mock.send(&mail).await.unwrap();
/// assert_eq!(mock.sent().len(), 1);
/// ```
#[derive(Debug)]
pub struct MockEmailClient {
    sent: Mutex<Vec<Mail>>,
}

impl MockEmailClient {
    /// Create a new mock with an empty send history.
    pub fn new() -> Self {
        Self {
            sent: Mutex::new(Vec::new()),
        }
    }

    /// Returns a snapshot of all mails that have been sent through this mock.
    pub fn sent(&self) -> Vec<Mail> {
        self.sent.lock().expect("mock email lock poisoned").clone()
    }

    /// Returns the most recently sent mail, or `None` if nothing has been sent.
    pub fn latest(&self) -> Option<Mail> {
        self.sent
            .lock()
            .expect("mock email lock poisoned")
            .last()
            .cloned()
    }
}

impl Default for MockEmailClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailClient for MockEmailClient {
    fn send<'a>(
        &'a self,
        mail: &'a Mail,
    ) -> Pin<Box<dyn Future<Output = Result<(), EmailError>> + Send + 'a>> {
        self.sent
            .lock()
            .expect("mock email lock poisoned")
            .push(mail.clone());
        Box::pin(async { Ok(()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email::{MailType, Recipient};

    #[tokio::test]
    async fn records_sent_mail() {
        let mock = MockEmailClient::new();
        let mail = Mail {
            recipient: Recipient {
                name: "alice".into(),
                email: "alice@example.com".into(),
            },
            mail_type: MailType::EmailVerification { token: "t1".into() },
        };

        mock.send(&mail).await.unwrap();
        mock.send(&mail).await.unwrap();

        assert_eq!(mock.sent().len(), 2);
    }

    #[tokio::test]
    async fn starts_empty() {
        let mock = MockEmailClient::new();
        assert!(mock.sent().is_empty());
        assert!(mock.latest().is_none());
    }

    #[tokio::test]
    async fn latest_returns_most_recent() {
        let mock = MockEmailClient::new();

        let first = Mail {
            recipient: Recipient {
                name: "alice".into(),
                email: "alice@example.com".into(),
            },
            mail_type: MailType::EmailVerification { token: "t1".into() },
        };
        let second = Mail {
            recipient: Recipient {
                name: "bob".into(),
                email: "bob@example.com".into(),
            },
            mail_type: MailType::PasswordReset { token: "t2".into() },
        };

        mock.send(&first).await.unwrap();
        mock.send(&second).await.unwrap();

        let latest = mock.latest().expect("should have a latest mail");
        assert_eq!(latest.recipient.email, "bob@example.com");
    }
}
