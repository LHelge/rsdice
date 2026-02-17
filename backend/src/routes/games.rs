use crate::{games::Game, prelude::*, repositories::UserRepository};
use axum::{
    Json, Router,
    extract::{Path, State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::get,
};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_games).put(create_game))
        .route("/{id}", get(get_game))
        .route("/{id}/ws", get(ws_handler))
}

async fn list_games(State(state): State<AppState>) -> Json<Vec<Game>> {
    Json(state.games.list_games().await)
}

async fn create_game(State(state): State<AppState>, claims: Claims) -> Result<Json<Game>> {
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(claims.sub).await?.ok_or(Error::NotFound)?;
    let creator = user.into();

    let world = common::World::from_string(include_str!("../../worlds/default.world"));
    let game = state.games.create_game(world, creator).await;
    Ok(Json(game))
}

async fn get_game(State(state): State<AppState>, Path(id): Path<uuid::Uuid>) -> Result<Json<Game>> {
    let game = state.games.get_game(&id).await.ok_or(Error::NotFound)?;
    Ok(Json(game))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    claims: Claims,
) -> Result<impl IntoResponse> {
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(claims.sub).await?.ok_or(Error::NotFound)?;
    let player_name = user.username.clone();

    let game = state.games.get_game(&id).await.ok_or(Error::NotFound)?;
    game.join_player(user.id, player_name).await?;

    Ok(ws.on_upgrade(move |socket| handle_game_socket(socket, user.id, game)))
}

async fn handle_game_socket(socket: WebSocket, user_id: Uuid, game: Game) {
    // TODO: implement WebSocket handling for real-time game updates and player actions
}
