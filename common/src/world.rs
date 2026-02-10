use crate::{AttackError, Stack};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Area {
    pub id: Uuid,
    pub owner: Option<Uuid>,
    pub tiles: HashSet<Tile>,
    pub stack: Stack,
}

impl Area {
    pub fn new(tiles: HashSet<Tile>) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner: None,
            tiles,
            stack: Stack::default(),
        }
    }

    pub fn center(&self) -> (f32, f32) {
        let (sum_x, sum_y): (f32, f32) = self
            .tiles
            .iter()
            .map(|tile| tile.to_world_coordinates())
            .fold((0.0, 0.0), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));

        let count = self.tiles.len() as f32;
        if count > 0.0 {
            (sum_x / count, sum_y / count)
        } else {
            (0.0, 0.0)
        }
    }

    pub fn is_owned_by(&self, player_id: Uuid) -> bool {
        self.owner == Some(player_id)
    }

    pub fn is_adjacent(&self, other: &Area) -> bool {
        self.tiles.iter().any(|tile| {
            other
                .tiles
                .iter()
                .any(|other_tile| tile.is_adjacent(other_tile))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct World {
    pub areas: HashMap<Uuid, Area>,
}

impl World {
    pub fn validate_attack(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        player_id: Uuid,
    ) -> Result<(), AttackError> {
        let from_area = self
            .areas
            .get(&from_id)
            .ok_or(AttackError::AreaNotFound(from_id))?;
        let to_area = self
            .areas
            .get(&to_id)
            .ok_or(AttackError::AreaNotFound(to_id))?;

        if !from_area.is_adjacent(to_area) {
            return Err(AttackError::AreasNotAdjacent(from_id, to_id));
        }

        if !from_area.is_owned_by(player_id) {
            return Err(AttackError::AreaNotOwnedByPlayer(from_id, player_id));
        }

        if to_area.is_owned_by(player_id) {
            return Err(AttackError::SelfAttackNotAllowed);
        }

        if from_area.stack.is_single() {
            return Err(AttackError::AreaNotEnoughDice(from_id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a Tile.
    fn tile(x: usize, y: usize) -> Tile {
        Tile::new(x, y)
    }

    /// Helper: create an Area with a single tile.
    fn area_with_tile(x: usize, y: usize) -> Area {
        let mut tiles = HashSet::new();
        tiles.insert(tile(x, y));
        Area::new(tiles)
    }

    /// Helper: create an Area with the given tiles.
    fn area_with_tiles(coords: &[(usize, usize)]) -> Area {
        let tiles: HashSet<Tile> = coords.iter().map(|&(x, y)| tile(x, y)).collect();
        Area::new(tiles)
    }

    /// Helper: build a World from a vec of areas.
    fn world_from_areas(areas: Vec<Area>) -> World {
        let map: HashMap<Uuid, Area> = areas.into_iter().map(|a| (a.id, a)).collect();
        World { areas: map }
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

    #[test]
    fn tile_hash_equal_tiles() {
        let mut set = HashSet::new();
        set.insert(tile(3, 4));
        set.insert(tile(3, 4));
        assert_eq!(set.len(), 1);
    }

    // ==== Tile serialization ====

    #[test]
    fn tile_serialize_deserialize_roundtrip() {
        let t = tile(5, 10);
        let json = serde_json::to_string(&t).unwrap();
        let deser: Tile = serde_json::from_str(&json).unwrap();
        assert_eq!(t, deser);
    }

    // ================================================================
    // ==== Area ====
    // ================================================================

    // ==== Area::new ====

    #[test]
    fn area_new_has_no_owner() {
        let area = area_with_tile(0, 0);
        assert!(area.owner.is_none());
    }

    #[test]
    fn area_new_has_default_stack() {
        let area = area_with_tile(0, 0);
        assert_eq!(area.stack.count(), Stack::MIN);
    }

    #[test]
    fn area_new_stores_tiles() {
        let area = area_with_tiles(&[(0, 0), (1, 0), (0, 1)]);
        assert_eq!(area.tiles.len(), 3);
    }

    #[test]
    fn area_new_has_unique_id() {
        let a = area_with_tile(0, 0);
        let b = area_with_tile(0, 0);
        assert_ne!(a.id, b.id);
    }

    // ==== Area::center ====

    #[test]
    fn area_center_single_tile() {
        let area = area_with_tile(0, 0);
        let (cx, cy) = area.center();
        let (wx, wy) = tile(0, 0).to_world_coordinates();
        assert!((cx - wx).abs() < f32::EPSILON);
        assert!((cy - wy).abs() < f32::EPSILON);
    }

    #[test]
    fn area_center_empty_tiles() {
        let area = Area::new(HashSet::new());
        let (cx, cy) = area.center();
        assert!((cx - 0.0).abs() < f32::EPSILON);
        assert!((cy - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn area_center_multiple_tiles_is_average() {
        let area = area_with_tiles(&[(0, 0), (1, 0)]);
        let (w0x, w0y) = tile(0, 0).to_world_coordinates();
        let (w1x, w1y) = tile(1, 0).to_world_coordinates();
        let (cx, cy) = area.center();
        assert!((cx - (w0x + w1x) / 2.0).abs() < f32::EPSILON);
        assert!((cy - (w0y + w1y) / 2.0).abs() < f32::EPSILON);
    }

    // ==== Area::is_owned_by ====

    #[test]
    fn area_is_owned_by_correct_player() {
        let mut area = area_with_tile(0, 0);
        let player_id = Uuid::new_v4();
        area.owner = Some(player_id);
        assert!(area.is_owned_by(player_id));
    }

    #[test]
    fn area_is_not_owned_by_other_player() {
        let mut area = area_with_tile(0, 0);
        area.owner = Some(Uuid::new_v4());
        assert!(!area.is_owned_by(Uuid::new_v4()));
    }

    #[test]
    fn area_is_not_owned_when_no_owner() {
        let area = area_with_tile(0, 0);
        assert!(!area.is_owned_by(Uuid::new_v4()));
    }

    // ==== Area::is_adjacent ====

    #[test]
    fn adjacent_areas_share_adjacent_tiles() {
        let a = area_with_tile(0, 0);
        let b = area_with_tile(0, 1);
        assert!(a.is_adjacent(&b));
    }

    #[test]
    fn non_adjacent_areas_with_distant_tiles() {
        let a = area_with_tile(0, 0);
        let b = area_with_tile(10, 10);
        assert!(!a.is_adjacent(&b));
    }

    #[test]
    fn area_adjacency_is_symmetric() {
        let a = area_with_tile(0, 0);
        let b = area_with_tile(0, 1);
        assert_eq!(a.is_adjacent(&b), b.is_adjacent(&a));
    }

    #[test]
    fn area_with_multiple_tiles_adjacent_if_any_tile_adjacent() {
        let a = area_with_tiles(&[(0, 0), (0, 1)]);
        let b = area_with_tiles(&[(0, 2), (0, 3)]);
        // (0,1) and (0,2) are adjacent
        assert!(a.is_adjacent(&b));
    }

    #[test]
    fn area_with_multiple_tiles_not_adjacent_if_none_adjacent() {
        let a = area_with_tiles(&[(0, 0), (0, 1)]);
        let b = area_with_tiles(&[(0, 3), (0, 4)]);
        assert!(!a.is_adjacent(&b));
    }

    // ==== Area serialization ====

    #[test]
    fn area_serialize_deserialize_roundtrip() {
        let mut area = area_with_tiles(&[(0, 0), (1, 0)]);
        area.owner = Some(Uuid::new_v4());
        area.stack.increment().unwrap();
        let json = serde_json::to_string(&area).unwrap();
        let deser: Area = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.id, area.id);
        assert_eq!(deser.owner, area.owner);
        assert_eq!(deser.tiles.len(), area.tiles.len());
        assert_eq!(deser.stack.count(), area.stack.count());
    }

    // ================================================================
    // ==== World::validate_attack ====
    // ================================================================

    // ==== Valid attack ====

    #[test]
    fn validate_attack_succeeds_for_valid_attack() {
        let player = Uuid::new_v4();
        let enemy = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(player);
        from.stack.increment().unwrap(); // 2 dice

        let mut to = area_with_tile(0, 1); // adjacent
        to.owner = Some(enemy);

        let world = world_from_areas(vec![from.clone(), to.clone()]);
        assert!(world.validate_attack(from.id, to.id, player).is_ok());
    }

    // ==== Area not found ====

    #[test]
    fn validate_attack_from_area_not_found() {
        let player = Uuid::new_v4();
        let fake_id = Uuid::new_v4();

        let mut to = area_with_tile(0, 1);
        to.owner = Some(Uuid::new_v4());

        let world = world_from_areas(vec![to.clone()]);
        let err = world.validate_attack(fake_id, to.id, player).unwrap_err();
        assert!(matches!(err, AttackError::AreaNotFound(id) if id == fake_id));
    }

    #[test]
    fn validate_attack_to_area_not_found() {
        let player = Uuid::new_v4();
        let fake_id = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(player);
        from.stack.increment().unwrap();

        let world = world_from_areas(vec![from.clone()]);
        let err = world.validate_attack(from.id, fake_id, player).unwrap_err();
        assert!(matches!(err, AttackError::AreaNotFound(id) if id == fake_id));
    }

    // ==== Areas not adjacent ====

    #[test]
    fn validate_attack_areas_not_adjacent() {
        let player = Uuid::new_v4();
        let enemy = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(player);
        from.stack.increment().unwrap();

        let mut to = area_with_tile(10, 10); // far away
        to.owner = Some(enemy);

        let world = world_from_areas(vec![from.clone(), to.clone()]);
        let err = world.validate_attack(from.id, to.id, player).unwrap_err();
        assert!(matches!(err, AttackError::AreasNotAdjacent(_, _)));
    }

    // ==== Area not owned by player ====

    #[test]
    fn validate_attack_from_area_not_owned_by_player() {
        let player = Uuid::new_v4();
        let other = Uuid::new_v4();
        let enemy = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(other); // owned by someone else
        from.stack.increment().unwrap();

        let mut to = area_with_tile(0, 1);
        to.owner = Some(enemy);

        let world = world_from_areas(vec![from.clone(), to.clone()]);
        let err = world.validate_attack(from.id, to.id, player).unwrap_err();
        assert!(matches!(err, AttackError::AreaNotOwnedByPlayer(_, _)));
    }

    // ==== Self attack ====

    #[test]
    fn validate_attack_self_attack_not_allowed() {
        let player = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(player);
        from.stack.increment().unwrap();

        let mut to = area_with_tile(0, 1);
        to.owner = Some(player); // same owner

        let world = world_from_areas(vec![from.clone(), to.clone()]);
        let err = world.validate_attack(from.id, to.id, player).unwrap_err();
        assert!(matches!(err, AttackError::SelfAttackNotAllowed));
    }

    // ==== Not enough dice ====

    #[test]
    fn validate_attack_not_enough_dice() {
        let player = Uuid::new_v4();
        let enemy = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(player);
        // stack is default (1 die) — not enough to attack

        let mut to = area_with_tile(0, 1);
        to.owner = Some(enemy);

        let world = world_from_areas(vec![from.clone(), to.clone()]);
        let err = world.validate_attack(from.id, to.id, player).unwrap_err();
        assert!(matches!(err, AttackError::AreaNotEnoughDice(_)));
    }

    // ==== Attack against unowned area ====

    #[test]
    fn validate_attack_against_unowned_area_succeeds() {
        let player = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(player);
        from.stack.increment().unwrap();

        let to = area_with_tile(0, 1); // no owner

        let world = world_from_areas(vec![from.clone(), to.clone()]);
        assert!(world.validate_attack(from.id, to.id, player).is_ok());
    }

    // ==== Error display messages ====

    #[test]
    fn attack_error_area_not_found_message() {
        let id = Uuid::new_v4();
        let err = AttackError::AreaNotFound(id);
        assert_eq!(err.to_string(), format!("area with ID {id} does not exist"));
    }

    #[test]
    fn attack_error_areas_not_adjacent_message() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let err = AttackError::AreasNotAdjacent(a, b);
        assert_eq!(
            err.to_string(),
            format!("areas with IDs {a} and {b} are not adjacent")
        );
    }

    #[test]
    fn attack_error_not_owned_message() {
        let area = Uuid::new_v4();
        let player = Uuid::new_v4();
        let err = AttackError::AreaNotOwnedByPlayer(area, player);
        assert_eq!(
            err.to_string(),
            format!("area with ID {area} is not owned by player with ID {player}")
        );
    }

    #[test]
    fn attack_error_not_enough_dice_message() {
        let id = Uuid::new_v4();
        let err = AttackError::AreaNotEnoughDice(id);
        assert_eq!(
            err.to_string(),
            format!("area with ID {id} does not have enough dice to attack")
        );
    }

    #[test]
    fn attack_error_self_attack_message() {
        let err = AttackError::SelfAttackNotAllowed;
        assert_eq!(err.to_string(), "a player cannot attack their own area");
    }

    // ==== World default ====

    #[test]
    fn default_world_has_no_areas() {
        let world = World::default();
        assert!(world.areas.is_empty());
    }
}
