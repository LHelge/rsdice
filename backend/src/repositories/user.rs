use crate::models::{User, UserError};
use crate::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository<'a> {
    db: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }
    /// Create a new user in the database.
    pub async fn create(&self, username: &str, password: &str, admin: bool) -> Result<User> {
        let user = User::new(username, password, admin)?;

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, password_hash, admin)
            VALUES ($1, $2, $3, $4)
            "#,
            user.id,
            user.username,
            user.password_hash,
            user.admin,
        )
        .execute(self.db)
        .await
        .map_err(|e| {
            // Check for unique constraint violation (TODO: This is specific to Postgres, consider using a more database-agnostic approach if needed)
            if let sqlx::Error::Database(ref db_err) = e
                && db_err
                    .message()
                    .contains("duplicate key value violates unique constraint")
            {
                return Error::User(UserError::UsernameExists);
            }
            Error::Database(e)
        })?;

        Ok(user)
    }

    /// Find a user by their ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, password_hash, admin as "admin: bool"
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
            SELECT id, username, password_hash, admin as "admin: bool"
            FROM users
            WHERE username = $1
            "#,
            username,
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
            SELECT id, username, password_hash, admin as "admin: bool"
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
