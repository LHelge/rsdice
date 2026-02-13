use serde::{Deserialize, Serialize};

// The tile grid is defined with the top-left corner as (0, 0) and the bottom-right corner as (width-1, height-1).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    x: usize,
    y: usize,
}

impl Tile {
    const SIZE: f32 = 1.0; // Size of each tile in world units

    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    // Converts tile coordinates to world coordinates (center of the tile)
    // For hexagonal tiles, we need to account for the staggered rows. Odd rows are offset by half a tile width.
    pub fn to_world_coordinates(&self) -> (f32, f32) {
        (
            self.x as f32 * Self::SIZE + Self::SIZE / 2.0,
            self.y as f32 * Self::SIZE
                + Self::SIZE / 2.0
                + if self.x.is_multiple_of(2) {
                    Self::SIZE / 2.0
                } else {
                    0.0
                },
        )
    }

    pub fn is_adjacent(&self, other: &Tile) -> bool {
        let dx = (self.x as isize - other.x as isize).abs();
        let dy = (self.y as isize - other.y as isize).abs();

        // For hexagonal tiles, two tiles are adjacent if they are next to each other in any of the 6 directions
        (dx == 1 && dy == 0)
            || (dx == 1 && dy == 1 && self.x.is_multiple_of(2))
            || (dx == 1 && dy == -1 && !self.x.is_multiple_of(2))
            || (dx == 0 && dy == 1)
            || (dx == 0 && dy == -1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a Tile.
    fn tile(x: usize, y: usize) -> Tile {
        Tile::new(x, y)
    }

    // ================================================================
    // ==== Tile ====
    // ================================================================

    // ==== Tile::new ====

    #[test]
    fn tile_new_stores_coordinates() {
        let t = tile(3, 7);
        assert_eq!(t.x, 3);
        assert_eq!(t.y, 7);
    }

    // ==== Tile::to_world_coordinates ====

    #[test]
    fn tile_world_coordinates_origin() {
        let (wx, wy) = tile(0, 0).to_world_coordinates();
        // x=0 is even, so offset of SIZE/2 on y
        assert!((wx - 0.5).abs() < f32::EPSILON);
        assert!((wy - 1.0).abs() < f32::EPSILON); // 0 + 0.5 + 0.5 = 1.0
    }

    #[test]
    fn tile_world_coordinates_odd_column() {
        let (wx, wy) = tile(1, 0).to_world_coordinates();
        // x=1 is odd, no extra y offset
        assert!((wx - 1.5).abs() < f32::EPSILON);
        assert!((wy - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn tile_world_coordinates_even_column() {
        let (wx, wy) = tile(2, 3).to_world_coordinates();
        // x=2 is even, offset
        assert!((wx - 2.5).abs() < f32::EPSILON);
        assert!((wy - 4.0).abs() < f32::EPSILON); // 3 + 0.5 + 0.5
    }

    // ==== Tile::is_adjacent ====

    #[test]
    fn tile_adjacent_same_column_above() {
        assert!(tile(0, 1).is_adjacent(&tile(0, 0)));
    }

    #[test]
    fn tile_adjacent_same_column_below() {
        assert!(tile(0, 0).is_adjacent(&tile(0, 1)));
    }

    #[test]
    fn tile_not_adjacent_to_self() {
        assert!(!tile(2, 2).is_adjacent(&tile(2, 2)));
    }

    #[test]
    fn tile_not_adjacent_far_away() {
        assert!(!tile(0, 0).is_adjacent(&tile(5, 5)));
    }

    #[test]
    fn tile_adjacent_horizontal() {
        // (1,0) and (2,0): dx=1, dy=0 → always adjacent
        assert!(tile(1, 0).is_adjacent(&tile(2, 0)));
    }

    #[test]
    fn tile_not_adjacent_two_apart_same_column() {
        assert!(!tile(0, 0).is_adjacent(&tile(0, 2)));
    }

    #[test]
    fn tile_adjacency_is_symmetric() {
        let a = tile(1, 1);
        let b = tile(2, 1);
        assert_eq!(a.is_adjacent(&b), b.is_adjacent(&a));
    }

    // ==== Tile equality & hashing ====

    #[test]
    fn tile_equality() {
        assert_eq!(tile(1, 2), tile(1, 2));
        assert_ne!(tile(1, 2), tile(2, 1));
    }
}
