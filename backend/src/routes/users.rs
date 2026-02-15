use crate::{
    email::{Mail, MailType, Recipient},
    models::{User, UserError},
    prelude::*,
    repositories::UserRepository,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::Duration;
use serde::Deserialize;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me))
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/password", post(update_password))
        .route("/auth", post(authenticate))
        .route("/register", post(register))
        .route("/resend-verification", post(resend_verification))
        .route("/verify-email", post(verify_email))
        .route("/logout", post(logout))
}

async fn send_verification_email(
    state: &AppState,
    repo: &UserRepository<'_>,
    user: &User,
) -> Result<()> {
    let token = repo.create_email_verification_token(user.id).await?;

    let mail = Mail {
        recipient: Recipient {
            name: user.username.clone(),
            email: user.email.clone(),
        },
        mail_type: MailType::EmailVerification { token },
    };

    state.email.send(&mail).await?;
    Ok(())
}

/// Get the current authenticated user.
async fn me(State(state): State<AppState>, claims: Claims) -> Result<Json<User>> {
    debug!(user_id = %claims.sub, "Fetching current user profile");
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(claims.sub).await?.ok_or(Error::NotFound)?;
    debug!(user_id = %claims.sub, "Current user profile fetched");
    Ok(Json(user))
}

/// List all users (admin only).
async fn list_users(State(state): State<AppState>, claims: Claims) -> Result<Json<Vec<User>>> {
    debug!(requester_id = %claims.sub, is_admin = claims.admin, "Listing users requested");
    if !claims.admin {
        warn!(requester_id = %claims.sub, "Non-admin attempted to list users");
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let users = repo.find_all().await?;
    debug!(requester_id = %claims.sub, user_count = users.len(), "Listed users");
    Ok(Json(users))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    password: String,
    #[serde(default)]
    admin: bool,
}

/// Create a new user (admin only).
async fn create_user(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>> {
    info!(requester_id = %claims.sub, username = %payload.username, admin = payload.admin, "Create user requested");
    if !claims.admin {
        warn!(requester_id = %claims.sub, "Non-admin attempted to create user");
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let user = repo
        .create(
            &payload.username,
            &payload.email,
            &payload.password,
            payload.admin,
        )
        .await?;
    info!(requester_id = %claims.sub, user_id = %user.id, username = %user.username, "User created by admin");
    Ok(Json(user))
}

/// Get a user by ID.
async fn get_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<User>> {
    debug!(requester_id = %claims.sub, target_user_id = %id, is_admin = claims.admin, "Get user requested");
    // Users can only view themselves unless they're admin
    if claims.sub != id && !claims.admin {
        warn!(requester_id = %claims.sub, target_user_id = %id, "Unauthorized user read attempt");
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(id).await?.ok_or(Error::NotFound)?;
    debug!(requester_id = %claims.sub, target_user_id = %id, "Get user succeeded");
    Ok(Json(user))
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    username: String,
    #[serde(default)]
    admin: bool,
}

/// Update a user's information (admin only).
async fn update_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>> {
    info!(requester_id = %claims.sub, target_user_id = %id, is_admin = claims.admin, "Update user requested");
    if !claims.admin {
        warn!(requester_id = %claims.sub, target_user_id = %id, "Non-admin attempted to update user");
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let user = repo
        .update(id, &payload.username, payload.admin)
        .await?
        .ok_or(Error::NotFound)?;
    info!(requester_id = %claims.sub, target_user_id = %id, username = %user.username, "User updated");
    Ok(Json(user))
}

#[derive(Deserialize)]
struct UpdatePasswordRequest {
    password: String,
}

/// Update a user's password (user can update their own, admin can update any).
async fn update_password(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Result<()> {
    info!(requester_id = %claims.sub, target_user_id = %id, is_admin = claims.admin, "Update password requested");
    if claims.sub != id && !claims.admin {
        warn!(requester_id = %claims.sub, target_user_id = %id, "Unauthorized password update attempt");
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let updated = repo.update_password(id, &payload.password).await?;
    if !updated {
        warn!(requester_id = %claims.sub, target_user_id = %id, "Password update target not found");
        return Err(Error::NotFound);
    }
    info!(requester_id = %claims.sub, target_user_id = %id, "Password updated");
    Ok(())
}

/// Delete a user (admin only).
async fn delete_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<()> {
    info!(requester_id = %claims.sub, target_user_id = %id, is_admin = claims.admin, "Delete user requested");
    if !claims.admin {
        warn!(requester_id = %claims.sub, target_user_id = %id, "Non-admin attempted to delete user");
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let deleted = repo.delete(id).await?;
    if !deleted {
        warn!(requester_id = %claims.sub, target_user_id = %id, "Delete target not found");
        return Err(Error::NotFound);
    }
    info!(requester_id = %claims.sub, target_user_id = %id, "User deleted");
    Ok(())
}

#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

/// Authenticate a user and return a JWT token in a cookie.
async fn authenticate(
    cookies: CookieJar,
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<(CookieJar, Json<User>)> {
    debug!(username = %payload.username, "Authentication attempt");
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_username(&payload.username)
        .await?
        .ok_or(Error::NotFound)?;

    user.verify_password(&payload.password)?;
    info!(user_id = %user.id, username = %user.username, "Authentication succeeded");

    let token = Claims::new(user.id, user.admin, &user.username, Duration::hours(1))
        .encode(&state.config.jwt_secret)?;

    // TODO: Set secure flag in production
    let cookie = Cookie::build(("token", token))
        .same_site(SameSite::Lax)
        .http_only(true)
        .path("/")
        .build();

    Ok((cookies.add(cookie), Json(user)))
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

/// Register a new user (public endpoint).
async fn register(
    cookies: CookieJar,
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(CookieJar, Json<User>)> {
    info!(username = %payload.username, email = %payload.email, "User registration requested");
    let repo = UserRepository::new(&state.db);
    let user = repo
        .create(&payload.username, &payload.email, &payload.password, false)
        .await?;
    info!(user_id = %user.id, username = %user.username, "User registered");

    send_verification_email(&state, &repo, &user).await?;
    info!(user_id = %user.id, "Verification email sent");

    let token = Claims::new(user.id, user.admin, &user.username, Duration::hours(1))
        .encode(&state.config.jwt_secret)?;

    // TODO: Set secure flag in production
    let cookie = Cookie::build(("token", token))
        .same_site(SameSite::Lax)
        .http_only(true)
        .path("/")
        .build();

    Ok((cookies.add(cookie), Json(user)))
}

/// Resend a verification email for the authenticated user.
async fn resend_verification(State(state): State<AppState>, claims: Claims) -> Result<()> {
    info!(user_id = %claims.sub, "Resend verification requested");
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(claims.sub).await?.ok_or(Error::NotFound)?;

    if user.email_verified {
        warn!(user_id = %claims.sub, "Resend verification skipped for already verified user");
        return Err(Error::User(UserError::EmailAlreadyVerified));
    }

    send_verification_email(&state, &repo, &user).await?;
    info!(user_id = %claims.sub, "Verification email resent");

    Ok(())
}

#[derive(Deserialize)]
struct VerifyEmailRequest {
    token: String,
}

/// Verifies a user's email address from a one-time token.
async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<()> {
    debug!(
        token_length = payload.token.trim().len(),
        "Verify email requested"
    );
    let repo = UserRepository::new(&state.db);
    let verified = repo.verify_email_token(payload.token.trim()).await?;

    if !verified {
        warn!("Email verification failed due to invalid or expired token");
        return Err(Error::User(UserError::InvalidVerificationToken));
    }

    info!("Email verification succeeded");

    Ok(())
}

/// Logout the current user by clearing the token cookie.
async fn logout(cookies: CookieJar) -> CookieJar {
    debug!("Logout requested");
    cookies.remove(Cookie::from("token"))
}
