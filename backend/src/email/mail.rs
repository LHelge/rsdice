use askama::Template;

/// Email recipient identity.
#[derive(Debug, Clone)]
pub struct Recipient {
    /// Display name (e.g. username).
    pub name: String,
    /// Email address.
    pub email: String,
}

/// The kind of email to send, carrying any type-specific data.
#[derive(Debug, Clone)]
pub enum MailType {
    /// Account verification email sent after registration.
    EmailVerification { token: String },
    /// Password-reset email.
    #[allow(dead_code)]
    PasswordReset { token: String },
}

/// An outbound application email.
///
/// Combines a [`Recipient`] with a [`MailType`] that determines the subject,
/// HTML body, and plain-text body. Call [`Mail::subject`], [`Mail::to_html`],
/// and [`Mail::to_text`] to produce the final content.
#[derive(Debug, Clone)]
pub struct Mail {
    /// Who the email is addressed to.
    pub recipient: Recipient,
    /// What kind of email to send.
    pub mail_type: MailType,
}

// ============================================================================
// Askama templates
// ============================================================================

#[derive(Template)]
#[template(path = "verification_email.html")]
struct VerificationEmailTemplate<'a> {
    username: &'a str,
    verification_url: &'a str,
}

#[derive(Template)]
#[template(path = "password_reset_email.html")]
struct PasswordResetEmailTemplate<'a> {
    username: &'a str,
    reset_url: &'a str,
}

impl Mail {
    /// Email subject line.
    pub fn subject(&self) -> &str {
        match &self.mail_type {
            MailType::EmailVerification { .. } => "Verify your rsdice account",
            MailType::PasswordReset { .. } => "Reset your rsdice password",
        }
    }

    /// Render the HTML body.
    ///
    /// `base_url` is the application's public URL (e.g. `https://rsdice.example.com`)
    /// and is used to construct action links inside the email.
    pub fn to_html(&self, base_url: &str) -> Result<String, askama::Error> {
        let base = base_url.trim_end_matches('/');
        let username = &self.recipient.name;

        match &self.mail_type {
            MailType::EmailVerification { token } => {
                let verification_url = format!("{base}/verify-email?token={token}");
                let template = VerificationEmailTemplate {
                    username,
                    verification_url: &verification_url,
                };
                template.render()
            }
            MailType::PasswordReset { token } => {
                let reset_url = format!("{base}/reset-password?token={token}");
                let template = PasswordResetEmailTemplate {
                    username,
                    reset_url: &reset_url,
                };
                template.render()
            }
        }
    }

    /// Render a plain-text body.
    pub fn to_text(&self, base_url: &str) -> String {
        let base = base_url.trim_end_matches('/');
        let username = &self.recipient.name;

        match &self.mail_type {
            MailType::EmailVerification { token } => {
                let url = format!("{base}/verify-email?token={token}");
                format!(
                    "Hi {username},\n\n\
                     Please verify your rsdice account by clicking the link below:\n\
                     {url}\n\n\
                     If you did not create this account, you can ignore this email."
                )
            }
            MailType::PasswordReset { token } => {
                let url = format!("{base}/reset-password?token={token}");
                format!(
                    "Hi {username},\n\n\
                     We received a request to reset your rsdice password.\n\
                     Click the link below to choose a new password:\n\
                     {url}\n\n\
                     If you did not request this, you can ignore this email."
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verification_mail() -> Mail {
        Mail {
            recipient: Recipient {
                name: "alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            mail_type: MailType::EmailVerification {
                token: "abc123".to_string(),
            },
        }
    }

    fn reset_mail() -> Mail {
        Mail {
            recipient: Recipient {
                name: "bob".to_string(),
                email: "bob@example.com".to_string(),
            },
            mail_type: MailType::PasswordReset {
                token: "xyz789".to_string(),
            },
        }
    }

    // ==== Subject ====

    #[test]
    fn verification_subject() {
        assert_eq!(verification_mail().subject(), "Verify your rsdice account");
    }

    #[test]
    fn reset_subject() {
        assert_eq!(reset_mail().subject(), "Reset your rsdice password");
    }

    // ==== Recipient ====

    #[test]
    fn verification_recipient() {
        let mail = verification_mail();
        assert_eq!(mail.recipient.email, "alice@example.com");
        assert_eq!(mail.recipient.name, "alice");
    }

    #[test]
    fn reset_recipient() {
        let mail = reset_mail();
        assert_eq!(mail.recipient.email, "bob@example.com");
        assert_eq!(mail.recipient.name, "bob");
    }

    // ==== Plain text ====

    #[test]
    fn verification_text_contains_url() {
        let text = verification_mail().to_text("https://rsdice.example.com");
        assert!(text.contains("https://rsdice.example.com/verify-email?token=abc123"));
        assert!(text.contains("alice"));
    }

    #[test]
    fn reset_text_contains_url() {
        let text = reset_mail().to_text("https://rsdice.example.com/");
        assert!(text.contains("https://rsdice.example.com/reset-password?token=xyz789"));
        assert!(text.contains("bob"));
    }

    #[test]
    fn text_trims_trailing_slash() {
        let text = verification_mail().to_text("https://rsdice.example.com/");
        // Should not produce a double slash before the path
        assert!(!text.contains(".com//"));
    }

    // ==== HTML ====

    #[test]
    fn verification_html_renders() {
        let html = verification_mail()
            .to_html("https://rsdice.example.com")
            .unwrap();
        assert!(html.contains("alice"));
        assert!(html.contains("https://rsdice.example.com/verify-email?token=abc123"));
    }

    #[test]
    fn reset_html_renders() {
        let html = reset_mail().to_html("https://rsdice.example.com").unwrap();
        assert!(html.contains("bob"));
        assert!(html.contains("https://rsdice.example.com/reset-password?token=xyz789"));
    }
}
