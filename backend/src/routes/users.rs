use crate::{models::User, prelude::*, repositories::UserRepository};
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
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me))
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/password", post(update_password))
        .route("/auth", post(authenticate))
        .route("/register", post(register))
        .route("/logout", post(logout))
}

/// Get the current authenticated user.
async fn me(State(state): State<AppState>, claims: Claims) -> Result<Json<User>> {
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(claims.sub).await?.ok_or(Error::NotFound)?;
    Ok(Json(user))
}

/// List all users (admin only).
async fn list_users(State(state): State<AppState>, claims: Claims) -> Result<Json<Vec<User>>> {
    if !claims.admin {
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let users = repo.find_all().await?;
    Ok(Json(users))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
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
    if !claims.admin {
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let user = repo
        .create(&payload.username, &payload.password, payload.admin)
        .await?;
    Ok(Json(user))
}

/// Get a user by ID.
async fn get_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<User>> {
    // Users can only view themselves unless they're admin
    if claims.sub != id && !claims.admin {
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(id).await?.ok_or(Error::NotFound)?;
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
    if !claims.admin {
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let user = repo
        .update(id, &payload.username, payload.admin)
        .await?
        .ok_or(Error::NotFound)?;
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
) -> Result<Json<()>> {
    if claims.sub != id && !claims.admin {
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let updated = repo.update_password(id, &payload.password).await?;
    if !updated {
        return Err(Error::NotFound);
    }
    Ok(Json(()))
}

/// Delete a user (admin only).
async fn delete_user(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    if !claims.admin {
        return Err(Error::NotFound);
    }
    let repo = UserRepository::new(&state.db);
    let deleted = repo.delete(id).await?;
    if !deleted {
        return Err(Error::NotFound);
    }
    Ok(Json(()))
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
    let repo = UserRepository::new(&state.db);
    let user = repo
        .find_by_username(&payload.username)
        .await?
        .ok_or(Error::NotFound)?;

    user.verify_password(&payload.password)?;

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
    password: String,
}

/// Register a new user (public endpoint).
async fn register(
    cookies: CookieJar,
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(CookieJar, Json<User>)> {
    let repo = UserRepository::new(&state.db);
    let user = repo
        .create(&payload.username, &payload.password, false)
        .await?;

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

/// Logout the current user by clearing the token cookie.
async fn logout(cookies: CookieJar) -> CookieJar {
    cookies.remove(Cookie::from("token"))
}
