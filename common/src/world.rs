use crate::{Area, AttackError};
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct World {
    pub areas: HashMap<Uuid, Area>,
}

impl World {
    pub fn from_string(s: &str) -> Self {
        let mut areas = HashMap::new();
        for line in s.lines() {
            let mut tiles = HashSet::new();
            for tile_str in line.split_whitespace() {
                if let Some((x_str, y_str)) = tile_str.split_once(',')
                    && let (Ok(x), Ok(y)) = (x_str.parse(), y_str.parse())
                {
                    tiles.insert(crate::Tile::new(x, y));
                }
            }
            if !tiles.is_empty() {
                let area = Area::new(tiles);
                areas.insert(area.id, area);
            }
        }
        Self { areas }
    }

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

    pub fn largest_connected_group(&self, player_id: Uuid) -> usize {
        let mut visited = HashSet::new();
        let mut largest = 0;

        for area in self.areas.values() {
            if area.is_owned_by(player_id) && !visited.contains(&area.id) {
                let size = self.dfs(area.id, player_id, &mut visited);
                largest = largest.max(size);
            }
        }

        largest
    }

    /// Depth-first traversal counting how many of `player_id`'s areas are
    /// reachable from the area with `start_id` via adjacency.
    fn dfs(&self, start_id: Uuid, player_id: Uuid, visited: &mut HashSet<Uuid>) -> usize {
        visited.insert(start_id);
        let mut size = 1;

        let start_area = match self.areas.get(&start_id) {
            Some(a) => a,
            None => return size,
        };

        for other in self.areas.values() {
            if !visited.contains(&other.id)
                && other.is_owned_by(player_id)
                && start_area.is_adjacent(other)
            {
                size += self.dfs(other.id, player_id, visited);
            }
        }

        size
    }

    /// Add a single die to a random non-full area owned by `player_id`.
    /// Returns `true` if a die was placed, `false` if the player has no areas
    /// or all of their areas are already at maximum dice.
    pub fn add_bonus_dice(&mut self, player_id: Uuid) -> bool {
        let eligible_ids: Vec<Uuid> = self
            .areas
            .values()
            .filter(|a| a.is_owned_by(player_id) && !a.stack.is_full())
            .map(|a| a.id)
            .collect();

        let Some(&chosen_id) = eligible_ids.choose(&mut rand::rng()) else {
            return false;
        };

        if let Some(area) = self.areas.get_mut(&chosen_id) {
            // increment is safe because we filtered out full stacks
            let _ = area.stack.increment();
        }

        true
    }

    pub fn is_winner(&self, player_id: Uuid) -> bool {
        self.areas
            .values()
            .filter(|area| !area.is_owned_by(player_id))
            .all(|area| area.is_not_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tile;

    fn area_with_tile(x: usize, y: usize) -> Area {
        let mut tiles = HashSet::new();
        tiles.insert(Tile::new(x, y));
        Area::new(tiles)
    }

    fn world_from_areas(areas: Vec<Area>) -> World {
        let map: HashMap<Uuid, Area> = areas.into_iter().map(|a| (a.id, a)).collect();
        World { areas: map }
    }

    #[test]
    fn validate_attack_happy_path() {
        let attacker = Uuid::new_v4();
        let defender = Uuid::new_v4();

        let mut from = area_with_tile(0, 0);
        from.owner = Some(attacker);
        from.stack.increment().unwrap();

        let mut to = area_with_tile(0, 1);
        to.owner = Some(defender);

        let world = world_from_areas(vec![from.clone(), to.clone()]);
        assert!(world.validate_attack(from.id, to.id, attacker).is_ok());
    }

    #[test]
    fn largest_connected_group_counts_owned_cluster() {
        let player = Uuid::new_v4();

        let mut a1 = area_with_tile(0, 0);
        a1.owner = Some(player);
        let mut a2 = area_with_tile(0, 1);
        a2.owner = Some(player);
        let mut a3 = area_with_tile(3, 3);
        a3.owner = Some(player);

        let world = world_from_areas(vec![a1, a2, a3]);
        assert_eq!(world.largest_connected_group(player), 2);
    }

    #[test]
    fn add_bonus_dice_returns_false_when_no_eligible_areas() {
        let player = Uuid::new_v4();
        let mut area = area_with_tile(0, 0);
        area.owner = Some(player);
        for _ in 1..crate::Stack::MAX {
            area.stack.increment().unwrap();
        }

        let mut world = world_from_areas(vec![area]);
        assert!(!world.add_bonus_dice(player));
    }

    #[test]
    fn is_winner_true_when_others_unowned() {
        let player = Uuid::new_v4();
        let mut mine = area_with_tile(0, 0);
        mine.owner = Some(player);
        let other = area_with_tile(1, 0);

        let world = world_from_areas(vec![mine, other]);
        assert!(world.is_winner(player));
    }

    #[test]
    fn from_string_parses_valid_tiles() {
        let input = "0,0 1,1\n2,2 3,3";
        let world = World::from_string(input);

        assert_eq!(world.areas.len(), 2);

        for area in world.areas.values() {
            assert!(area.tiles.len() == 2);
        }
    }

    #[test]
    fn from_string_ignores_invalid_tile_format() {
        let input = "0,0 invalid 1,1\nno_comma";
        let world = World::from_string(input);

        // Should create an area only for lines containing at least one valid tile
        assert_eq!(world.areas.len(), 1);
    }

    #[test]
    fn from_string_skips_empty_lines() {
        let input = "0,0 1,1\n\n2,2 3,3";
        let world = World::from_string(input);

        assert_eq!(world.areas.len(), 2);
    }

    #[test]
    fn from_string_creates_no_areas_for_empty_input() {
        let input = "";
        let world = World::from_string(input);

        assert_eq!(world.areas.len(), 0);
    }

    #[test]
    fn from_string_creates_one_area_per_line() {
        let input = "0,0 1,0\n2,0 3,0\n4,0 5,0";
        let world = World::from_string(input);

        assert_eq!(world.areas.len(), 3);
    }
}
