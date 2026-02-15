use crate::models::{User, UserError};
use crate::prelude::*;
use chrono::Duration;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository<'a> {
    db: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    fn hash_verification_token(token: &str) -> String {
        let digest = Sha256::digest(token.as_bytes());
        format!("{digest:x}")
    }

    fn hash_refresh_token(token: &str) -> String {
        let digest = Sha256::digest(token.as_bytes());
        format!("{digest:x}")
    }

    fn hash_password_reset_token(token: &str) -> String {
        let digest = Sha256::digest(token.as_bytes());
        format!("{digest:x}")
    }

    /// Create a new user in the database.
    pub async fn create(
        &self,
        username: &str,
        email: &str,
        password: &str,
        admin: bool,
    ) -> Result<User> {
        let user = User::new(username, email, password, admin)?;

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, email_verified, admin)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.email_verified,
            user.admin,
        )
        .execute(self.db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if matches!(db_err.constraint(), Some("users_username_key")) {
                    return Error::User(UserError::UsernameExists);
                }

                if matches!(db_err.constraint(), Some("users_email_key")) {
                    return Error::User(UserError::EmailExists);
                }
            }

            Error::Database(e)
        })?;

        Ok(user)
    }

    /// Creates a one-time email verification token and returns the raw token.
    pub async fn create_email_verification_token(&self, user_id: Uuid) -> Result<String> {
        let token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
        let token_hash = Self::hash_verification_token(&token);

        sqlx::query!(
            r#"
            DELETE FROM email_verification_tokens
            WHERE user_id = $1
              AND used_at IS NULL
            "#,
            user_id,
        )
        .execute(self.db)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at)
            VALUES ($1, $2, $3, NOW() + INTERVAL '24 HOURS')
            "#,
            Uuid::new_v4(),
            user_id,
            token_hash,
        )
        .execute(self.db)
        .await?;

        Ok(token)
    }

    /// Verifies an email token and marks the corresponding user as verified.
    pub async fn verify_email_token(&self, token: &str) -> Result<bool> {
        let token_hash = Self::hash_verification_token(token);

        let token_row = sqlx::query!(
            r#"
            UPDATE email_verification_tokens
            SET used_at = NOW()
            WHERE token_hash = $1
              AND used_at IS NULL
              AND expires_at > NOW()
            RETURNING user_id
            "#,
            token_hash,
        )
        .fetch_optional(self.db)
        .await?;

        let Some(row) = token_row else {
            return Ok(false);
        };

        sqlx::query!(
            r#"
            UPDATE users
            SET email_verified = TRUE
            WHERE id = $1
            "#,
            row.user_id,
        )
        .execute(self.db)
        .await?;

        Ok(true)
    }

    /// Creates and stores a refresh token, returning the raw token.
    pub async fn create_refresh_token(&self, user_id: Uuid, lifetime: Duration) -> Result<String> {
        let token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
        let token_hash = Self::hash_refresh_token(&token);

        sqlx::query!(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
            VALUES ($1, $2, $3, NOW() + ($4 * INTERVAL '1 second'))
            "#,
            Uuid::new_v4(),
            user_id,
            token_hash,
            lifetime.num_seconds() as f64,
        )
        .execute(self.db)
        .await?;

        Ok(token)
    }

    /// Revokes all active refresh tokens for a user.
    pub async fn revoke_all_refresh_tokens(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1
              AND revoked_at IS NULL
            "#,
            user_id,
        )
        .execute(self.db)
        .await?;

        Ok(())
    }

    /// Creates a one-time password reset token and returns the raw token.
    pub async fn create_password_reset_token(&self, user_id: Uuid) -> Result<String> {
        let token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
        let token_hash = Self::hash_password_reset_token(&token);

        sqlx::query!(
            r#"
            DELETE FROM password_reset_tokens
            WHERE user_id = $1
              AND used_at IS NULL
            "#,
            user_id,
        )
        .execute(self.db)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at)
            VALUES ($1, $2, $3, NOW() + INTERVAL '24 HOURS')
            "#,
            Uuid::new_v4(),
            user_id,
            token_hash,
        )
        .execute(self.db)
        .await?;

        Ok(token)
    }

    /// Consumes a password reset token and updates the user's password.
    pub async fn consume_password_reset_token(&self, token: &str, password: &str) -> Result<bool> {
        let token_hash = Self::hash_password_reset_token(token);
        let mut transaction = self.db.begin().await?;

        let token_row = sqlx::query!(
            r#"
            UPDATE password_reset_tokens
            SET used_at = NOW()
            WHERE token_hash = $1
              AND used_at IS NULL
              AND expires_at > NOW()
            RETURNING user_id
            "#,
            token_hash,
        )
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(row) = token_row else {
            transaction.rollback().await?;
            return Ok(false);
        };

        let password_hash = User::hash_password(password)?;

        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE id = $2
            "#,
            password_hash,
            row.user_id,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1
              AND revoked_at IS NULL
            "#,
            row.user_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(true)
    }

    /// Returns the user id of a valid refresh token.
    pub async fn validate_refresh_token(&self, token: &str) -> Result<Option<Uuid>> {
        let token_hash = Self::hash_refresh_token(token);

        let row = sqlx::query!(
            r#"
            SELECT user_id
            FROM refresh_tokens
            WHERE token_hash = $1
              AND revoked_at IS NULL
              AND expires_at > NOW()
            "#,
            token_hash,
        )
        .fetch_optional(self.db)
        .await?;

        Ok(row.map(|record| record.user_id))
    }

    /// Revokes a refresh token by value.
    pub async fn revoke_refresh_token(&self, token: &str) -> Result<bool> {
        let token_hash = Self::hash_refresh_token(token);

        let result = sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE token_hash = $1
              AND revoked_at IS NULL
            "#,
            token_hash,
        )
        .execute(self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Find a user by their ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, email_verified as "email_verified: bool", admin as "admin: bool"
            FROM users
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(self.db)
        .await?;

        Ok(user)
    }

    /// Find a user by their username.
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, email_verified as "email_verified: bool", admin as "admin: bool"
            FROM users
            WHERE username = $1
            "#,
            username,
        )
        .fetch_optional(self.db)
        .await?;

        Ok(user)
    }

    /// Find a user by their email address.
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let normalized_email = email.trim().to_ascii_lowercase();

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, email_verified as "email_verified: bool", admin as "admin: bool"
            FROM users
            WHERE email = $1
            "#,
            normalized_email,
        )
        .fetch_optional(self.db)
        .await?;

        Ok(user)
    }

    /// Find a user by username or email.
    pub async fn find_by_username_or_email(&self, identifier: &str) -> Result<Option<User>> {
        let trimmed_identifier = identifier.trim();
        if trimmed_identifier.is_empty() {
            return Ok(None);
        }

        let normalized_email = trimmed_identifier.to_ascii_lowercase();

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, email_verified as "email_verified: bool", admin as "admin: bool"
            FROM users
            WHERE username = $1 OR email = $2
            "#,
            trimmed_identifier,
            normalized_email,
        )
        .fetch_optional(self.db)
        .await?;

        Ok(user)
    }

    /// Get all users from the database.
    pub async fn find_all(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, email_verified as "email_verified: bool", admin as "admin: bool"
            FROM users
            ORDER BY id
            "#,
        )
        .fetch_all(self.db)
        .await?;

        Ok(users)
    }

    /// Update a user's information.
    pub async fn update(&self, id: Uuid, username: &str, admin: bool) -> Result<Option<User>> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET username = $1, admin = $2
            WHERE id = $3
            "#,
            username,
            admin,
            id,
        )
        .execute(self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.find_by_id(id).await
    }

    /// Update a user's password.
    pub async fn update_password(&self, id: Uuid, password: &str) -> Result<bool> {
        let password_hash = User::hash_password(password)?;

        let result = sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE id = $2
            "#,
            password_hash,
            id,
        )
        .execute(self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a user by their ID.
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id,
        )
        .execute(self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
