use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The maximum number of players allowed in a single game.
pub const MAX_PLAYERS: usize = 6;

/// Player colors. Each variant has a fixed numeric index (`#[repr(usize)]`)
/// and a hex color value accessible via [`Color::to_hex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(usize)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
    Purple = 4,
    Orange = 5,
}

impl Color {
    /// All color variants in index order.
    pub const ALL: [Color; MAX_PLAYERS] = [
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Yellow,
        Color::Purple,
        Color::Orange,
    ];

    /// Returns the hex color string (e.g. `"#FF0000"` for `Red`).
    pub fn to_hex(self) -> &'static str {
        match self {
            Color::Red => "#FF0000",
            Color::Green => "#00CC44",
            Color::Blue => "#3366FF",
            Color::Yellow => "#FFDD00",
            Color::Purple => "#9933FF",
            Color::Orange => "#FF8800",
        }
    }
}

impl From<Color> for usize {
    fn from(color: Color) -> Self {
        color as usize
    }
}

impl TryFrom<usize> for Color {
    type Error = ColorError;

    fn try_from(value: usize) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Color::Red),
            1 => Ok(Color::Green),
            2 => Ok(Color::Blue),
            3 => Ok(Color::Yellow),
            4 => Ok(Color::Purple),
            5 => Ok(Color::Orange),
            _ => Err(ColorError::InvalidIndex(value)),
        }
    }
}

/// Errors related to [`Color`] conversion.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ColorError {
    #[error("invalid color index {0}, expected 0–5")]
    InvalidIndex(usize),
}

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
        self.stored_dice = self
            .stored_dice
            .clamp(self.stored_dice + amount, Self::MAX_STORED_DICE);
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

    // ==== Color::to_hex ====

    #[test]
    fn to_hex_returns_correct_values() {
        assert_eq!(Color::Red.to_hex(), "#FF0000");
        assert_eq!(Color::Green.to_hex(), "#00CC44");
        assert_eq!(Color::Blue.to_hex(), "#3366FF");
        assert_eq!(Color::Yellow.to_hex(), "#FFDD00");
        assert_eq!(Color::Purple.to_hex(), "#9933FF");
        assert_eq!(Color::Orange.to_hex(), "#FF8800");
    }

    // ==== Numeric conversions ====

    #[test]
    fn color_to_usize() {
        assert_eq!(usize::from(Color::Red), 0);
        assert_eq!(usize::from(Color::Green), 1);
        assert_eq!(usize::from(Color::Blue), 2);
        assert_eq!(usize::from(Color::Yellow), 3);
        assert_eq!(usize::from(Color::Purple), 4);
        assert_eq!(usize::from(Color::Orange), 5);
    }

    #[test]
    fn usize_to_color_valid() {
        for i in 0usize..=5 {
            let color = Color::try_from(i).unwrap();
            assert_eq!(usize::from(color), i);
        }
    }

    #[test]
    fn usize_to_color_invalid() {
        assert!(Color::try_from(6usize).is_err());
        assert!(Color::try_from(255usize).is_err());
    }

    // ==== Color::ALL ====

    #[test]
    fn all_contains_all_variants_in_order() {
        assert_eq!(Color::ALL.len(), MAX_PLAYERS);
        for (i, color) in Color::ALL.iter().enumerate() {
            assert_eq!(usize::from(*color), i);
        }
    }

    // ==== MAX_PLAYERS ====

    #[test]
    fn max_players_is_six() {
        assert_eq!(MAX_PLAYERS, 6);
    }

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
