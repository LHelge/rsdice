use argon2::{
    Argon2, PasswordHash,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] argon2::password_hash::Error),

    #[error(
        "Weak password, it must be at least 10 characters and contain a mix of letters, numbers, and symbols."
    )]
    WeakPassword,

    #[error("Username must be at least 3 characters.")]
    UsernameTooShort,

    #[error("Username already exists.")]
    UsernameExists,

    #[error("Email is invalid.")]
    InvalidEmail,

    #[error("Email already exists.")]
    EmailExists,

    #[error("Invalid or expired verification token.")]
    InvalidVerificationToken,

    #[error("Invalid or expired password reset token.")]
    InvalidPasswordResetToken,

    #[error("Email is already verified.")]
    EmailAlreadyVerified,
}

pub type Result<T> = std::result::Result<T, UserError>;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,

    pub username: String,

    pub email: String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    #[serde(default)]
    pub email_verified: bool,

    pub admin: bool,
}

impl User {
    pub fn new(
        username: impl Into<String>,
        email: impl Into<String>,
        password: &str,
        admin: bool,
    ) -> Result<Self> {
        let username = username.into();
        let email = email.into().trim().to_ascii_lowercase();
        Self::validate_username(&username)?;
        Self::validate_email(&email)?;
        Self::validate_password(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            email_verified: false,
            admin,
        })
    }

    pub fn verify_password(&self, password: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
        Ok(())
    }

    /// Hash a password without creating a full User.
    /// Useful for password updates where we only need the hash.
    pub fn hash_password(password: &str) -> Result<String> {
        Self::validate_password(password)?;
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    fn validate_username(username: &str) -> Result<()> {
        if username.len() < 3 {
            return Err(UserError::UsernameTooShort);
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<()> {
        if !EmailAddress::is_valid(email) {
            return Err(UserError::InvalidEmail);
        }

        Ok(())
    }

    fn validate_password(password: &str) -> Result<()> {
        if password.len() < 10 {
            return Err(UserError::WeakPassword);
        }
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

        if has_upper && has_lower && has_digit && has_symbol {
            Ok(())
        } else {
            Err(UserError::WeakPassword)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Password Validation Tests
    // ========================================================================

    #[test]
    fn validate_rejects_short_password() {
        let result = User::new("testuser", "test@example.com", "Ab1!", false);
        assert!(matches!(result, Err(UserError::WeakPassword)));
    }

    #[test]
    fn validate_rejects_password_without_uppercase() {
        let result = User::new("testuser", "test@example.com", "abcdefgh1!", false);
        assert!(matches!(result, Err(UserError::WeakPassword)));
    }

    #[test]
    fn validate_rejects_password_without_lowercase() {
        let result = User::new("testuser", "test@example.com", "ABCDEFGH1!", false);
        assert!(matches!(result, Err(UserError::WeakPassword)));
    }

    #[test]
    fn validate_rejects_password_without_digit() {
        let result = User::new("testuser", "test@example.com", "Abcdefghij!", false);
        assert!(matches!(result, Err(UserError::WeakPassword)));
    }

    #[test]
    fn validate_rejects_password_without_symbol() {
        let result = User::new("testuser", "test@example.com", "Abcdefghij1", false);
        assert!(matches!(result, Err(UserError::WeakPassword)));
    }

    #[test]
    fn validate_accepts_strong_password() {
        let result = User::new("testuser", "test@example.com", "Abcdefgh1!", false);
        assert!(result.is_ok());
    }

    // ========================================================================
    // User Creation Tests
    // ========================================================================

    #[test]
    fn new_creates_user_with_hashed_password() {
        let user = User::new("testuser", "test@example.com", "Abcdefgh1!", false).unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.email_verified);
        assert!(!user.admin);
        // Password should be hashed, not plaintext
        assert_ne!(user.password_hash, "Abcdefgh1!");
        assert!(user.password_hash.starts_with("$argon2"));
    }

    #[test]
    fn new_creates_admin_user() {
        let user = User::new("admin", "admin@example.com", "Abcdefgh1!", true).unwrap();
        assert!(user.admin);
    }

    #[test]
    fn new_accepts_string_username() {
        let username = String::from("testuser");
        let user = User::new(username, "test@example.com", "Abcdefgh1!", false).unwrap();
        assert_eq!(user.username, "testuser");
    }

    // ========================================================================
    // Password Verification Tests
    // ========================================================================

    #[test]
    fn verify_password_succeeds_with_correct_password() {
        let user = User::new("testuser", "test@example.com", "Abcdefgh1!", false).unwrap();
        assert!(user.verify_password("Abcdefgh1!").is_ok());
    }

    #[test]
    fn verify_password_fails_with_wrong_password() {
        let user = User::new("testuser", "test@example.com", "Abcdefgh1!", false).unwrap();
        assert!(user.verify_password("WrongPassword1!").is_err());
    }

    #[test]
    fn verify_password_fails_with_empty_password() {
        let user = User::new("testuser", "test@example.com", "Abcdefgh1!", false).unwrap();
        assert!(user.verify_password("").is_err());
    }

    #[test]
    fn verify_password_is_case_sensitive() {
        let user = User::new("testuser", "test@example.com", "Abcdefgh1!", false).unwrap();
        assert!(user.verify_password("abcdefgh1!").is_err());
    }

    // ========================================================================
    // Username Validation Tests
    // ========================================================================

    #[test]
    fn validate_rejects_empty_username() {
        let result = User::new("", "test@example.com", "Abcdefgh1!", false);
        assert!(matches!(result, Err(UserError::UsernameTooShort)));
    }

    #[test]
    fn validate_rejects_single_char_username() {
        let result = User::new("a", "test@example.com", "Abcdefgh1!", false);
        assert!(matches!(result, Err(UserError::UsernameTooShort)));
    }

    #[test]
    fn validate_rejects_two_char_username() {
        let result = User::new("ab", "test@example.com", "Abcdefgh1!", false);
        assert!(matches!(result, Err(UserError::UsernameTooShort)));
    }

    #[test]
    fn validate_accepts_three_char_username() {
        let result = User::new("abc", "test@example.com", "Abcdefgh1!", false);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_accepts_long_username() {
        let result = User::new("averylongusername", "test@example.com", "Abcdefgh1!", false);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Email Validation Tests
    // ========================================================================

    #[test]
    fn validate_rejects_invalid_email() {
        let result = User::new("testuser", "invalid-email", "Abcdefgh1!", false);
        assert!(matches!(result, Err(UserError::InvalidEmail)));
    }

    #[test]
    fn validate_accepts_email_with_mixed_case_and_normalizes() {
        let user = User::new("testuser", "TeSt@Example.COM", "Abcdefgh1!", false).unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn validate_rejects_email_with_whitespace() {
        let result = User::new("testuser", "test @example.com", "Abcdefgh1!", false);
        assert!(matches!(result, Err(UserError::InvalidEmail)));
    }

    #[test]
    fn validate_accepts_valid_email() {
        let result = User::new("testuser", "hello.world@example.com", "Abcdefgh1!", false);
        assert!(result.is_ok());
    }
}
