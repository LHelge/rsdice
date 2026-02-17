use crate::models::User;
use crate::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct Game {
    pub id: Uuid,

    #[allow(dead_code)] // TODO: remove when we have actual fields here
    #[serde(skip)]
    inner: Arc<RwLock<common::Game>>,
    // TODO: flatten fields from common::Game into this struct when serializing,
    // probablubly by implementing `Serialize` manually and using `inner.read().await`
    // to access the inner game state.
    pub creator: Creator,
}

impl Game {
    pub fn new(world: common::World, creator: Creator) -> Self {
        let inner = common::Game::new(world);
        Self {
            id: inner.id,
            inner: Arc::new(RwLock::new(inner)),
            creator,
            // Initialize other fields based on config
        }
    }

    pub async fn join_player(&self, player_id: Uuid, player_name: String) -> Result<()> {
        self.inner
            .write()
            .await
            .join_player(player_id, player_name)?;
        Ok(())
    }

    // Add methods to manipulate game state, e.g.:
    // - add_player
    // - remove_player
    // - make_move
    // - etc.
}
