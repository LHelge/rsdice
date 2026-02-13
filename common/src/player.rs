use crate::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub color: Color,
    stored_dice: usize,
}

impl Player {
    const MAX_STORED_DICE: usize = 20;

    pub fn new(id: Uuid, name: String, color: Color) -> Self {
        Self {
            id,
            name,
            color,
            stored_dice: 0,
        }
    }

    pub fn store_dice(&mut self, amount: usize) {
        self.stored_dice = (self.stored_dice + amount).clamp(0, Self::MAX_STORED_DICE);
    }

    pub fn take_stored_dice(&mut self) -> usize {
        let stored = self.stored_dice;
        self.stored_dice = 0;
        stored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==== Helpers ====

    fn make_player() -> Player {
        Player::new(Uuid::new_v4(), "Alice".into(), Color::Red)
    }

    // ==== store_dice ====

    #[test]
    fn new_player_has_zero_stored_dice() {
        let player = make_player();
        assert_eq!(player.stored_dice, 0);
    }

    #[test]
    fn store_dice_adds_amount() {
        let mut player = make_player();
        player.store_dice(5);
        assert_eq!(player.stored_dice, 5);
    }

    #[test]
    fn store_dice_accumulates() {
        let mut player = make_player();
        player.store_dice(3);
        player.store_dice(4);
        assert_eq!(player.stored_dice, 7);
    }

    #[test]
    fn store_dice_caps_at_max() {
        let mut player = make_player();
        player.store_dice(Player::MAX_STORED_DICE + 5);
        assert_eq!(player.stored_dice, Player::MAX_STORED_DICE);
    }

    #[test]
    fn store_dice_caps_across_multiple_calls() {
        let mut player = make_player();
        player.store_dice(15);
        player.store_dice(10);
        assert_eq!(player.stored_dice, Player::MAX_STORED_DICE);
    }

    #[test]
    fn store_dice_zero_is_noop() {
        let mut player = make_player();
        player.store_dice(3);
        player.store_dice(0);
        assert_eq!(player.stored_dice, 3);
    }

    // ==== take_stored_dice ====

    #[test]
    fn take_stored_dice_returns_zero_when_empty() {
        let mut player = make_player();
        assert_eq!(player.take_stored_dice(), 0);
    }

    #[test]
    fn take_stored_dice_returns_stored_amount() {
        let mut player = make_player();
        player.store_dice(7);
        assert_eq!(player.take_stored_dice(), 7);
    }

    #[test]
    fn take_stored_dice_resets_to_zero() {
        let mut player = make_player();
        player.store_dice(5);
        player.take_stored_dice();
        assert_eq!(player.stored_dice, 0);
    }

    #[test]
    fn take_stored_dice_second_call_returns_zero() {
        let mut player = make_player();
        player.store_dice(10);
        player.take_stored_dice();
        assert_eq!(player.take_stored_dice(), 0);
    }

    #[test]
    fn store_and_take_interleaved() {
        let mut player = make_player();
        player.store_dice(3);
        assert_eq!(player.take_stored_dice(), 3);
        player.store_dice(2);
        player.store_dice(4);
        assert_eq!(player.take_stored_dice(), 6);
    }
}
