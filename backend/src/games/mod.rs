mod game;

pub use game::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct Games {
    pub games: Arc<RwLock<HashMap<Uuid, Game>>>,
}

impl Games {
    pub async fn create_game(&self, world: common::World, creator: Creator) -> Game {
        let game = Game::new(world, creator);
        let game_id = game.id;
        self.games.write().await.insert(game_id, game.clone());
        game
    }

    pub async fn get_game(&self, game_id: &Uuid) -> Option<Game> {
        self.games.read().await.get(game_id).cloned()
    }

    pub async fn list_games(&self) -> Vec<Game> {
        self.games.read().await.values().cloned().collect()
    }
}
