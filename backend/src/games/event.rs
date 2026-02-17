use super::Creator;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameCommand {
    Start,
    Attack { from_id: Uuid, to_id: Uuid },
    EndTurn,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameEvent {
    Snapshot {
        game: common::Game,
    },
    PlayerJoined {
        player_id: Uuid,
        player_name: String,
    },
    GameStarted,
    AttackResolved {
        from_id: Uuid,
        to_id: Uuid,
        player_id: Uuid,
    },
    TurnEnded {
        player_id: Uuid,
    },
    Finished {
        reason: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameListItem {
    pub id: Uuid,
    pub creator: Creator,
    pub player_count: usize,
    pub state: common::GameState,
}
