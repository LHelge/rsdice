use super::{GameEvent, GameListItem};
use crate::models::User;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast, watch};
use uuid::Uuid;

const GAME_IDLE_TIMEOUT: Duration = Duration::from_secs(300);
const GAME_TIMEOUT_TICK: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creator {
    pub id: Uuid,
    pub name: String,
}

impl From<User> for Creator {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.username,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: Uuid,
    inner: Arc<RwLock<common::Game>>,
    snapshot_tx: watch::Sender<common::Game>,
    event_tx: broadcast::Sender<GameEvent>,
    activity_tx: watch::Sender<Instant>,
    pub creator: Creator,
}

impl Game {
    pub fn new(world: common::World, creator: Creator) -> Self {
        let inner = common::Game::new(world);
        let (snapshot_tx, _) = watch::channel(inner.clone());
        let (event_tx, _) = broadcast::channel(64);
        let (activity_tx, _) = watch::channel(Instant::now());

        let game = Self {
            id: inner.id,
            inner: Arc::new(RwLock::new(inner)),
            snapshot_tx,
            event_tx,
            activity_tx,
            creator,
        };

        game.spawn_timeout_task();
        game
    }

    pub async fn join_player(&self, player_id: Uuid, player_name: String) -> Result<()> {
        let event_name = player_name.clone();
        let snapshot = {
            let mut inner = self.inner.write().await;
            inner.join_player(player_id, player_name)?;
            inner.clone()
        };

        self.touch_activity();
        self.publish_event(GameEvent::PlayerJoined {
            player_id,
            player_name: event_name,
        });
        self.publish_snapshot(snapshot);
        Ok(())
    }

    pub async fn start_game(&self) -> Result<()> {
        let snapshot = {
            let mut inner = self.inner.write().await;
            inner.start()?;
            inner.clone()
        };

        self.touch_activity();
        self.publish_event(GameEvent::GameStarted);
        self.publish_snapshot(snapshot);
        Ok(())
    }

    pub async fn attack(&self, from_id: Uuid, to_id: Uuid, player_id: Uuid) -> Result<()> {
        let snapshot = {
            let mut inner = self.inner.write().await;
            inner.attack(from_id, to_id, player_id)?;
            inner.clone()
        };

        self.touch_activity();
        self.publish_event(GameEvent::AttackResolved {
            from_id,
            to_id,
            player_id,
        });
        self.publish_snapshot(snapshot);
        Ok(())
    }

    pub async fn end_turn(&self, player_id: Uuid) -> Result<()> {
        let snapshot = {
            let mut inner = self.inner.write().await;

            if let common::GameState::InProgress { turn } = inner.state
                && inner.players[turn].id != player_id
            {
                return Err(common::GameError::NotPlayerTurn.into());
            }

            inner.end_turn()?;
            inner.clone()
        };

        self.touch_activity();
        self.publish_event(GameEvent::TurnEnded { player_id });
        self.publish_snapshot(snapshot);
        Ok(())
    }

    pub fn touch_activity(&self) {
        let _ = self.activity_tx.send(Instant::now());
    }

    pub async fn snapshot(&self) -> common::Game {
        self.inner.read().await.clone()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<GameEvent> {
        self.event_tx.subscribe()
    }

    pub fn subscribe_snapshot(&self) -> watch::Receiver<common::Game> {
        self.snapshot_tx.subscribe()
    }

    pub async fn list_item(&self) -> GameListItem {
        let snapshot = self.snapshot().await;
        GameListItem {
            id: self.id,
            creator: self.creator.clone(),
            player_count: snapshot.players.len(),
            state: snapshot.state,
        }
    }

    fn spawn_timeout_task(&self) {
        let game = self.clone();
        tokio::spawn(async move {
            game.run_timeout_loop().await;
        });
    }

    async fn run_timeout_loop(self) {
        let mut ticker = tokio::time::interval(GAME_TIMEOUT_TICK);

        loop {
            ticker.tick().await;

            let last_activity = *self.activity_tx.borrow();
            if last_activity.elapsed() < GAME_IDLE_TIMEOUT {
                continue;
            }

            let timed_out_snapshot = {
                let mut inner = self.inner.write().await;
                if !matches!(inner.state, common::GameState::InProgress { .. }) {
                    continue;
                }

                inner.state = common::GameState::Finished;
                inner.clone()
            };

            self.publish_event(GameEvent::Finished {
                reason: "Game timed out due to inactivity".to_string(),
            });
            self.publish_snapshot(timed_out_snapshot);
            break;
        }
    }

    fn publish_snapshot(&self, snapshot: common::Game) {
        let _ = self.snapshot_tx.send(snapshot.clone());
        self.publish_event(GameEvent::Snapshot { game: snapshot });
    }

    fn publish_event(&self, event: GameEvent) {
        let _ = self.event_tx.send(event);
    }
}
