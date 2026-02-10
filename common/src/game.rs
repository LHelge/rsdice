use crate::{Color, ColorError, MAX_PLAYERS, StackError};

use super::{Player, World};
use rand::random_range;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum AttackError {
    #[error("area with ID {0} does not exist")]
    AreaNotFound(Uuid),

    #[error("areas with IDs {0} and {1} are not adjacent")]
    AreasNotAdjacent(Uuid, Uuid),

    #[error("area with ID {0} is not owned by player with ID {1}")]
    AreaNotOwnedByPlayer(Uuid, Uuid),

    #[error("area with ID {0} does not have enough dice to attack")]
    AreaNotEnoughDice(Uuid),

    #[error("a player cannot attack their own area")]
    SelfAttackNotAllowed,

    #[error("it's not the player's turn")]
    NotPlayerTurn,
}

/// Errors related to [`Game`] operations.
#[derive(Debug, Clone, Error)]
pub enum GameError {
    #[error("the game is already full")]
    GameFull,

    #[error("player is already in the game")]
    PlayerAlreadyInGame,

    #[error("it's not the player's turn")]
    NotPlayerTurn,

    #[error("the game has not started yet")]
    GameNotStarted,

    #[error("the game has already started")]
    GameStarted,

    #[error("the game has already finished")]
    GameFinished,

    #[error("not enough players to start the game")]
    NotEnoughPlayers,

    #[error("color conversion error: {0}")]
    ColorError(#[from] ColorError),

    #[error("attack validation error: {0}")]
    AttackError(#[from] AttackError),

    #[error("stack operation error: {0}")]
    StackError(#[from] StackError),
}

type Result<T> = std::result::Result<T, GameError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameState {
    WaitingForPlayers,
    InProgress { turn: usize },
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub world: World,
    pub players: Vec<Player>,
    pub state: GameState,
}

impl Game {
    pub fn new(world: World) -> Self {
        Self {
            id: Uuid::new_v4(),
            world,
            players: Vec::new(),
            state: GameState::WaitingForPlayers,
        }
    }

    pub fn join_player(&mut self, id: Uuid, name: String) -> Result<Player> {
        // Check if player is already in the game
        if self.players.iter().any(|p| p.id == id) {
            return Err(GameError::PlayerAlreadyInGame);
        }

        // Check if game is full
        if self.players.len() >= MAX_PLAYERS {
            return Err(GameError::GameFull);
        }

        if self.state != GameState::WaitingForPlayers {
            return Err(GameError::GameStarted);
        }

        let color = Color::try_from(self.players.len())?;
        let player = Player::new(id, name, color);
        self.players.push(player.clone());
        Ok(player)
    }

    pub fn start(&mut self) -> Result<()> {
        if self.state != GameState::WaitingForPlayers {
            return Err(GameError::GameStarted);
        }

        if self.players.len() < 2 {
            return Err(GameError::NotEnoughPlayers);
        }

        let first = random_range(..self.players.len());
        self.state = GameState::InProgress { turn: first };
        Ok(())
    }

    pub fn attack(&mut self, from_id: Uuid, to_id: Uuid, player_id: Uuid) -> Result<()> {
        // Validate attack
        self.world.validate_attack(from_id, to_id, player_id)?;

        // Borrow both areas mutably by temporarily removing the attacker's area
        let mut from_area = self
            .world
            .areas
            .remove(&from_id)
            .ok_or(AttackError::AreaNotFound(from_id))?;
        let to_area = self
            .world
            .areas
            .get_mut(&to_id)
            .ok_or(AttackError::AreaNotFound(to_id))?;

        let attack_roll = from_area.stack.attack_roll();
        let defense_roll = to_area.stack.defence_roll();

        if attack_roll > defense_roll {
            // Attacker wins: transfer ownership and move dice
            to_area.owner = Some(player_id);
            let (remaining_stack, moved_stack) = from_area.stack.split()?;
            to_area.stack = moved_stack;
            from_area.stack = remaining_stack;
        } else {
            // Defender wins: attacker loses all dice except one
            from_area.stack.defeat();
        }

        // Re-insert the attacking area
        self.world.areas.insert(from_id, from_area);

        Ok(())
    }

    pub fn next_turn(&mut self, player_id: Uuid) -> Result<()> {
        if self.state == GameState::Finished {
            return Err(GameError::GameFinished);
        }
        if self.state == GameState::WaitingForPlayers {
            return Err(GameError::GameNotStarted);
        }
        if let GameState::InProgress { turn } = &mut self.state {
            let current_player_id = self.players[*turn].id;
            if current_player_id != player_id {
                return Err(GameError::NotPlayerTurn);
            }
            *turn = (*turn + 1) % self.players.len();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Area, Stack, Tile};
    use std::collections::{HashMap, HashSet};

    /// Helper: create a World with no areas.
    fn empty_world() -> World {
        World::default()
    }

    /// Helper: create a game with an empty world.
    fn new_game() -> Game {
        Game::new(empty_world())
    }

    /// Helper: add N players to a game and return their IDs.
    fn add_players(game: &mut Game, n: usize) -> Vec<Uuid> {
        (0..n)
            .map(|i| {
                let id = Uuid::new_v4();
                game.join_player(id, format!("Player {i}")).unwrap();
                id
            })
            .collect()
    }

    /// Helper: build a World with two adjacent areas, returning (world, from_id, to_id).
    fn world_with_two_adjacent_areas(
        owner_from: Uuid,
        owner_to: Uuid,
        from_dice: usize,
    ) -> (World, Uuid, Uuid) {
        world_with_two_adjacent_areas_full(owner_from, owner_to, from_dice, 1)
    }

    /// Helper: build a World with two adjacent areas with configurable dice on both, returning (world, from_id, to_id).
    fn world_with_two_adjacent_areas_full(
        owner_from: Uuid,
        owner_to: Uuid,
        from_dice: usize,
        to_dice: usize,
    ) -> (World, Uuid, Uuid) {
        let mut tiles_a = HashSet::new();
        tiles_a.insert(Tile::new(0, 0));
        let mut from = Area::new(tiles_a);
        from.owner = Some(owner_from);
        for _ in 1..from_dice {
            from.stack.increment().unwrap();
        }

        let mut tiles_b = HashSet::new();
        tiles_b.insert(Tile::new(0, 1));
        let mut to = Area::new(tiles_b);
        to.owner = Some(owner_to);
        for _ in 1..to_dice {
            to.stack.increment().unwrap();
        }

        let from_id = from.id;
        let to_id = to.id;

        let mut areas = HashMap::new();
        areas.insert(from.id, from);
        areas.insert(to.id, to);

        (World { areas }, from_id, to_id)
    }

    // ================================================================
    // ==== Game::new ====
    // ================================================================

    #[test]
    fn new_game_starts_waiting_for_players() {
        let game = new_game();
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn new_game_has_no_players() {
        let game = new_game();
        assert!(game.players.is_empty());
    }

    #[test]
    fn new_game_has_unique_id() {
        let a = new_game();
        let b = new_game();
        assert_ne!(a.id, b.id);
    }

    // ================================================================
    // ==== Game::join_player ====
    // ================================================================

    #[test]
    fn join_player_adds_player() {
        let mut game = new_game();
        let id = Uuid::new_v4();
        let player = game.join_player(id, "Alice".into()).unwrap();
        assert_eq!(player.id, id);
        assert_eq!(player.name, "Alice");
        assert_eq!(game.players.len(), 1);
    }

    #[test]
    fn join_player_assigns_colors_in_order() {
        let mut game = new_game();
        let ids = add_players(&mut game, 6);
        for (i, player) in game.players.iter().enumerate() {
            assert_eq!(player.color, Color::ALL[i]);
            assert_eq!(player.id, ids[i]);
        }
    }

    #[test]
    fn join_player_duplicate_returns_error() {
        let mut game = new_game();
        let id = Uuid::new_v4();
        game.join_player(id, "Alice".into()).unwrap();
        let err = game.join_player(id, "Alice Again".into()).unwrap_err();
        assert!(matches!(err, GameError::PlayerAlreadyInGame));
    }

    #[test]
    fn join_player_game_full_returns_error() {
        let mut game = new_game();
        add_players(&mut game, MAX_PLAYERS);
        let err = game
            .join_player(Uuid::new_v4(), "Extra".into())
            .unwrap_err();
        assert!(matches!(err, GameError::GameFull));
    }

    #[test]
    fn join_player_after_game_started_returns_error() {
        let mut game = new_game();
        add_players(&mut game, 2);
        game.start().unwrap();
        let err = game.join_player(Uuid::new_v4(), "Late".into()).unwrap_err();
        assert!(matches!(err, GameError::GameStarted));
    }

    #[test]
    fn join_player_returns_correct_player_data() {
        let mut game = new_game();
        let id = Uuid::new_v4();
        let player = game.join_player(id, "Bob".into()).unwrap();
        assert_eq!(player.id, id);
        assert_eq!(player.name, "Bob");
        assert_eq!(player.color, Color::Red); // first player
    }

    #[test]
    fn join_second_player_gets_green() {
        let mut game = new_game();
        add_players(&mut game, 1);
        let id2 = Uuid::new_v4();
        let p2 = game.join_player(id2, "P2".into()).unwrap();
        assert_eq!(p2.color, Color::Green);
    }

    // ================================================================
    // ==== Game::start ====
    // ================================================================

    #[test]
    fn start_with_two_players_transitions_to_in_progress() {
        let mut game = new_game();
        add_players(&mut game, 2);
        game.start().unwrap();
        assert!(matches!(game.state, GameState::InProgress { .. }));
    }

    #[test]
    fn start_sets_turn_within_player_range() {
        let mut game = new_game();
        add_players(&mut game, 4);
        game.start().unwrap();
        if let GameState::InProgress { turn } = game.state {
            assert!(turn < 4);
        } else {
            panic!("expected InProgress state");
        }
    }

    #[test]
    fn start_with_no_players_returns_error() {
        let mut game = new_game();
        let err = game.start().unwrap_err();
        assert!(matches!(err, GameError::NotEnoughPlayers));
    }

    #[test]
    fn start_with_one_player_returns_error() {
        let mut game = new_game();
        add_players(&mut game, 1);
        let err = game.start().unwrap_err();
        assert!(matches!(err, GameError::NotEnoughPlayers));
    }

    #[test]
    fn start_already_started_returns_error() {
        let mut game = new_game();
        add_players(&mut game, 2);
        game.start().unwrap();
        let err = game.start().unwrap_err();
        assert!(matches!(err, GameError::GameStarted));
    }

    #[test]
    fn start_with_max_players() {
        let mut game = new_game();
        add_players(&mut game, MAX_PLAYERS);
        game.start().unwrap();
        assert!(matches!(game.state, GameState::InProgress { .. }));
    }

    // ================================================================
    // ==== Game::next_turn ====
    // ================================================================

    #[test]
    fn next_turn_advances_turn_index() {
        let mut game = new_game();
        let ids = add_players(&mut game, 3);
        game.start().unwrap();

        // Find whose turn it is
        let GameState::InProgress { turn } = game.state else {
            panic!("expected InProgress");
        };
        let current_player_id = ids[turn];
        let expected_next = (turn + 1) % 3;

        game.next_turn(current_player_id).unwrap();

        let GameState::InProgress { turn: new_turn } = game.state else {
            panic!("expected InProgress");
        };
        assert_eq!(new_turn, expected_next);
    }

    #[test]
    fn next_turn_wraps_around() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        // Force the state to have a deterministic turn
        game.state = GameState::InProgress { turn: 1 };

        game.next_turn(ids[1]).unwrap();
        assert_eq!(game.state, GameState::InProgress { turn: 0 });

        game.next_turn(ids[0]).unwrap();
        assert_eq!(game.state, GameState::InProgress { turn: 1 });
    }

    #[test]
    fn next_turn_wrong_player_returns_error() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        // Player 1 tries to end turn when it's player 0's turn
        let err = game.next_turn(ids[1]).unwrap_err();
        assert!(matches!(err, GameError::NotPlayerTurn));
    }

    #[test]
    fn next_turn_when_waiting_returns_error() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        let err = game.next_turn(ids[0]).unwrap_err();
        assert!(matches!(err, GameError::GameNotStarted));
    }

    #[test]
    fn next_turn_when_finished_returns_error() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::Finished;
        let err = game.next_turn(ids[0]).unwrap_err();
        assert!(matches!(err, GameError::GameFinished));
    }

    #[test]
    fn next_turn_full_cycle() {
        let mut game = new_game();
        let ids = add_players(&mut game, 4);
        game.state = GameState::InProgress { turn: 0 };

        for id in ids {
            game.next_turn(id).unwrap();
        }
        // After 4 next_turn calls with 4 players, we should be back to turn 0
        assert_eq!(game.state, GameState::InProgress { turn: 0 });
    }

    // ================================================================
    // ==== Game::attack ====
    // ================================================================

    #[test]
    fn attack_valid_succeeds() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        let (world, from_id, to_id) = world_with_two_adjacent_areas(ids[0], ids[1], 3);
        game.world = world;

        assert!(game.attack(from_id, to_id, ids[0]).is_ok());
    }

    #[test]
    fn attack_from_area_not_found() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        let err = game
            .attack(Uuid::new_v4(), Uuid::new_v4(), ids[0])
            .unwrap_err();
        assert!(matches!(err, GameError::AttackError(_)));
    }

    #[test]
    fn attack_not_enough_dice() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        let (world, from_id, to_id) = world_with_two_adjacent_areas(ids[0], ids[1], 1); // only 1 die
        game.world = world;

        let err = game.attack(from_id, to_id, ids[0]).unwrap_err();
        assert!(matches!(
            err,
            GameError::AttackError(AttackError::AreaNotEnoughDice(_))
        ));
    }

    #[test]
    fn attack_self_attack_not_allowed() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        let (world, from_id, to_id) = world_with_two_adjacent_areas(ids[0], ids[0], 3); // same owner
        game.world = world;

        let err = game.attack(from_id, to_id, ids[0]).unwrap_err();
        assert!(matches!(
            err,
            GameError::AttackError(AttackError::SelfAttackNotAllowed)
        ));
    }

    // ==== Attack outcome invariants ====

    #[test]
    fn attack_from_area_always_has_one_die_after_attack() {
        // Regardless of win or loss, the attacking area ends up with 1 die.
        for _ in 0..50 {
            let mut game = new_game();
            let ids = add_players(&mut game, 2);
            game.state = GameState::InProgress { turn: 0 };

            let (world, from_id, to_id) = world_with_two_adjacent_areas_full(ids[0], ids[1], 4, 2);
            game.world = world;

            game.attack(from_id, to_id, ids[0]).unwrap();

            let from_area = game.world.areas.get(&from_id).unwrap();
            assert_eq!(
                from_area.stack.count(),
                Stack::MIN,
                "attacker should always have 1 die after attacking"
            );
        }
    }

    #[test]
    fn attack_from_area_remains_in_world() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        let (world, from_id, to_id) = world_with_two_adjacent_areas(ids[0], ids[1], 3);
        game.world = world;

        game.attack(from_id, to_id, ids[0]).unwrap();

        assert!(
            game.world.areas.contains_key(&from_id),
            "from_area must be re-inserted into the world"
        );
        assert!(
            game.world.areas.contains_key(&to_id),
            "to_area must still exist"
        );
    }

    #[test]
    fn attack_from_area_ownership_unchanged() {
        // The attacker's area always stays owned by the attacker.
        for _ in 0..50 {
            let mut game = new_game();
            let ids = add_players(&mut game, 2);
            game.state = GameState::InProgress { turn: 0 };

            let (world, from_id, to_id) = world_with_two_adjacent_areas_full(ids[0], ids[1], 5, 3);
            game.world = world;

            game.attack(from_id, to_id, ids[0]).unwrap();

            let from_area = game.world.areas.get(&from_id).unwrap();
            assert_eq!(from_area.owner, Some(ids[0]));
        }
    }

    #[test]
    fn attack_win_transfers_ownership_and_dice() {
        // Use 8 attacker dice vs 1 defender die to almost guarantee a win.
        // Run many times to find at least one win.
        let mut saw_win = false;
        for _ in 0..200 {
            let mut game = new_game();
            let ids = add_players(&mut game, 2);
            game.state = GameState::InProgress { turn: 0 };

            let (world, from_id, to_id) = world_with_two_adjacent_areas_full(ids[0], ids[1], 8, 1);
            game.world = world;

            game.attack(from_id, to_id, ids[0]).unwrap();

            let to_area = game.world.areas.get(&to_id).unwrap();
            if to_area.owner == Some(ids[0]) {
                // Attacker won
                saw_win = true;
                // Captured area should have attacker's dice minus 1 (the remaining stack)
                assert_eq!(to_area.stack.count(), 8 - 1);
                break;
            }
        }
        assert!(
            saw_win,
            "with 8 vs 1 dice, attacker should win at least once in 200 tries"
        );
    }

    #[test]
    fn attack_loss_defender_area_unchanged() {
        // Use 2 attacker dice vs 8 defender dice to almost guarantee a loss.
        // Run many times to find at least one loss.
        let mut saw_loss = false;
        for _ in 0..200 {
            let mut game = new_game();
            let ids = add_players(&mut game, 2);
            game.state = GameState::InProgress { turn: 0 };

            let (world, from_id, to_id) = world_with_two_adjacent_areas_full(ids[0], ids[1], 2, 8);
            game.world = world;

            game.attack(from_id, to_id, ids[0]).unwrap();

            let to_area = game.world.areas.get(&to_id).unwrap();
            if to_area.owner == Some(ids[1]) {
                // Defender won — area should be unchanged
                saw_loss = true;
                assert_eq!(to_area.stack.count(), 8);
                assert_eq!(to_area.owner, Some(ids[1]));
                break;
            }
        }
        assert!(
            saw_loss,
            "with 2 vs 8 dice, defender should win at least once in 200 tries"
        );
    }

    #[test]
    fn attack_both_outcomes_possible() {
        // Use balanced dice (4 vs 4) and verify both outcomes occur.
        let mut wins = 0;
        let mut losses = 0;
        let iterations = 500;

        let player = Uuid::new_v4();
        let enemy = Uuid::new_v4();

        for _ in 0..iterations {
            let mut game = new_game();
            // Manually set up players with known IDs
            game.join_player(player, "P0".into()).unwrap();
            game.join_player(enemy, "P1".into()).unwrap();
            game.state = GameState::InProgress { turn: 0 };

            let (world, from_id, to_id) = world_with_two_adjacent_areas_full(player, enemy, 4, 4);
            game.world = world;

            game.attack(from_id, to_id, player).unwrap();

            let to_area = game.world.areas.get(&to_id).unwrap();
            if to_area.owner == Some(player) {
                wins += 1;
            } else {
                losses += 1;
            }
        }
        assert!(
            wins > 0,
            "expected at least one attacker win in {iterations} rounds"
        );
        assert!(
            losses > 0,
            "expected at least one defender win in {iterations} rounds"
        );
    }

    #[test]
    fn attack_preserves_world_area_count() {
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        let (world, from_id, to_id) = world_with_two_adjacent_areas_full(ids[0], ids[1], 4, 2);
        game.world = world;

        let area_count_before = game.world.areas.len();
        game.attack(from_id, to_id, ids[0]).unwrap();
        assert_eq!(
            game.world.areas.len(),
            area_count_before,
            "attack should not change the number of areas"
        );
    }

    #[test]
    fn attack_multiple_times_from_same_area_fails_after_dice_depleted() {
        // After one attack, from_area has 1 die and can't attack again.
        let mut game = new_game();
        let ids = add_players(&mut game, 2);
        game.state = GameState::InProgress { turn: 0 };

        let (world, from_id, to_id) = world_with_two_adjacent_areas_full(ids[0], ids[1], 3, 1);
        game.world = world;

        // First attack should succeed
        game.attack(from_id, to_id, ids[0]).unwrap();

        // If attacker won, to_id is now theirs — need a new target.
        // If attacker lost, from_id has 1 die. Either way, from_id has 1 die.
        let from_area = game.world.areas.get(&from_id).unwrap();
        assert_eq!(from_area.stack.count(), 1);

        // Attacking again from the same area should fail (not enough dice)
        // We need a valid enemy area adjacent. The to_area might now be ours, so just test the dice check.
        let to_area = game.world.areas.get(&to_id).unwrap();
        if to_area.owner != Some(ids[0]) {
            // Defender still owns it, so we can attempt again
            let err = game.attack(from_id, to_id, ids[0]).unwrap_err();
            assert!(matches!(
                err,
                GameError::AttackError(AttackError::AreaNotEnoughDice(_))
            ));
        }
    }

    // ================================================================
    // ==== GameState ====
    // ================================================================

    #[test]
    fn game_state_equality() {
        assert_eq!(GameState::WaitingForPlayers, GameState::WaitingForPlayers);
        assert_eq!(
            GameState::InProgress { turn: 0 },
            GameState::InProgress { turn: 0 }
        );
        assert_ne!(
            GameState::InProgress { turn: 0 },
            GameState::InProgress { turn: 1 }
        );
        assert_eq!(GameState::Finished, GameState::Finished);
        assert_ne!(GameState::WaitingForPlayers, GameState::Finished);
    }

    #[test]
    fn game_state_serialize_deserialize_roundtrip() {
        let states = vec![
            GameState::WaitingForPlayers,
            GameState::InProgress { turn: 3 },
            GameState::Finished,
        ];
        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deser: GameState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, deser);
        }
    }

    // ================================================================
    // ==== Game serialization ====
    // ================================================================

    #[test]
    fn game_serialize_deserialize_roundtrip() {
        let mut game = new_game();
        add_players(&mut game, 3);
        let json = serde_json::to_string(&game).unwrap();
        let deser: Game = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.id, game.id);
        assert_eq!(deser.players.len(), 3);
        assert_eq!(deser.state, GameState::WaitingForPlayers);
    }

    // ================================================================
    // ==== GameError display messages ====
    // ================================================================

    #[test]
    fn game_error_messages() {
        assert_eq!(GameError::GameFull.to_string(), "the game is already full");
        assert_eq!(
            GameError::PlayerAlreadyInGame.to_string(),
            "player is already in the game"
        );
        assert_eq!(
            GameError::NotPlayerTurn.to_string(),
            "it's not the player's turn"
        );
        assert_eq!(
            GameError::GameNotStarted.to_string(),
            "the game has not started yet"
        );
        assert_eq!(
            GameError::GameStarted.to_string(),
            "the game has already started"
        );
        assert_eq!(
            GameError::GameFinished.to_string(),
            "the game has already finished"
        );
        assert_eq!(
            GameError::NotEnoughPlayers.to_string(),
            "not enough players to start the game"
        );
    }
}
