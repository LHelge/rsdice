mod event;
mod game;

pub use event::*;
pub use game::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, watch};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Games {
    games: Arc<RwLock<HashMap<Uuid, Game>>>,
    list_tx: watch::Sender<Vec<GameListItem>>,
}

impl Default for Games {
    fn default() -> Self {
        let (list_tx, _) = watch::channel(Vec::new());
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            list_tx,
        }
    }
}

impl Games {
    pub async fn create_game(&self, world: common::World, creator: Creator) -> Game {
        let game = Game::new(world, creator);
        let game_id = game.id;
        self.games.write().await.insert(game_id, game.clone());

        let mut snapshots = game.subscribe_snapshot();
        let games = self.clone();
        tokio::spawn(async move {
            while snapshots.changed().await.is_ok() {
                games.publish_list_snapshot().await;
            }
        });

        self.publish_list_snapshot().await;
        game
    }

    pub async fn get_game(&self, game_id: &Uuid) -> Option<Game> {
        self.games.read().await.get(game_id).cloned()
    }

    pub async fn list_games(&self) -> Vec<GameListItem> {
        let games: Vec<Game> = self.games.read().await.values().cloned().collect();
        let mut out = Vec::with_capacity(games.len());

        for game in games {
            out.push(game.list_item().await);
        }

        out
    }

    pub fn subscribe_list(&self) -> watch::Receiver<Vec<GameListItem>> {
        self.list_tx.subscribe()
    }

    async fn publish_list_snapshot(&self) {
        let snapshot = self.list_games().await;
        let _ = self.list_tx.send(snapshot);
    }
}
