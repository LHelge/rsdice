use crate::{
    games::{Game, GameCommand, GameEvent, GameListItem},
    prelude::*,
    repositories::UserRepository,
};
use axum::{
    Json, Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use std::{convert::Infallible, time::Duration};
use tokio_stream::{StreamExt, wrappers::WatchStream};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_games).put(create_game))
        .route("/stream", get(list_games_sse))
        .route("/{id}", get(get_game))
        .route("/{id}/ws", get(game_ws))
}

async fn list_games(State(state): State<AppState>) -> Json<Vec<GameListItem>> {
    Json(state.games.list_games().await)
}

async fn create_game(State(state): State<AppState>, claims: Claims) -> Result<Json<common::Game>> {
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(claims.sub).await?.ok_or(Error::NotFound)?;
    let creator = user.into();

    let world = common::World::from_string(include_str!("../../worlds/default.world"));
    let game = state.games.create_game(world, creator).await;
    Ok(Json(game.snapshot().await))
}

async fn get_game(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<common::Game>> {
    let game = state.games.get_game(&id).await.ok_or(Error::NotFound)?;
    Ok(Json(game.snapshot().await))
}

async fn game_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    claims: Claims,
) -> Result<impl IntoResponse> {
    let repo = UserRepository::new(&state.db);
    let user = repo.find_by_id(claims.sub).await?.ok_or(Error::NotFound)?;
    let player_name = user.username.clone();

    let game = state.games.get_game(&id).await.ok_or(Error::NotFound)?;
    match game.join_player(user.id, player_name).await {
        Ok(()) => {}
        Err(Error::GameError(common::GameError::PlayerAlreadyInGame)) => {}
        Err(err) => return Err(err),
    }

    Ok(ws.on_upgrade(move |socket| handle_game_socket(socket, user.id, game)))
}

async fn handle_game_socket(mut socket: WebSocket, user_id: Uuid, game: Game) {
    if send_event(
        &mut socket,
        GameEvent::Snapshot {
            game: game.snapshot().await,
        },
    )
    .await
    .is_err()
    {
        return;
    }

    let mut events = game.subscribe_events();

    loop {
        tokio::select! {
            message = socket.recv() => {
                match message {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<GameCommand>(&text) {
                            Ok(command) => {
                                if let Err(err) = execute_command(&game, user_id, command).await {
                                    let _ = send_event(
                                        &mut socket,
                                        GameEvent::Error { message: err.to_string() }
                                    ).await;
                                }
                            }
                            Err(err) => {
                                let _ = send_event(
                                    &mut socket,
                                    GameEvent::Error { message: format!("invalid command payload: {err}") }
                                ).await;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        if send_event(&mut socket, event).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        if send_event(
                            &mut socket,
                            GameEvent::Snapshot { game: game.snapshot().await }
                        ).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn execute_command(game: &Game, user_id: Uuid, command: GameCommand) -> Result<()> {
    match command {
        GameCommand::Start => game.start_game().await,
        GameCommand::Attack { from_id, to_id } => game.attack(from_id, to_id, user_id).await,
        GameCommand::EndTurn => game.end_turn(user_id).await,
        GameCommand::Ping => {
            game.touch_activity();
            Ok(())
        }
    }
}

async fn list_games_sse(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>> {
    let stream = WatchStream::new(state.games.subscribe_list()).map(|games| {
        let data = serde_json::to_string(&games).unwrap_or_else(|_| "[]".to_string());
        Ok(Event::default().event("games").data(data))
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

async fn send_event(socket: &mut WebSocket, event: GameEvent) -> std::result::Result<(), ()> {
    let payload = serde_json::to_string(&event).map_err(|_| ())?;
    socket
        .send(Message::Text(payload.into()))
        .await
        .map_err(|_| ())
}
