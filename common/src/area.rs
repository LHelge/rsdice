use crate::{Stack, Tile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

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

    pub fn is_not_owned(&self) -> bool {
        self.owner.is_none()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn area_with_tiles(coords: &[(usize, usize)]) -> Area {
        let tiles: HashSet<Tile> = coords.iter().map(|&(x, y)| Tile::new(x, y)).collect();
        Area::new(tiles)
    }

    #[test]
    fn new_area_has_defaults() {
        let area = area_with_tiles(&[(0, 0)]);
        assert!(area.owner.is_none());
        assert_eq!(area.stack.count(), Stack::MIN);
    }

    #[test]
    fn center_returns_origin_for_empty_tiles() {
        let area = Area::new(HashSet::new());
        assert_eq!(area.center(), (0.0, 0.0));
    }

    #[test]
    fn is_owned_by_and_is_not_owned_work() {
        let player_id = Uuid::new_v4();
        let mut area = area_with_tiles(&[(0, 0)]);

        assert!(area.is_not_owned());
        assert!(!area.is_owned_by(player_id));

        area.owner = Some(player_id);
        assert!(!area.is_not_owned());
        assert!(area.is_owned_by(player_id));
    }

    #[test]
    fn adjacent_areas_detected() {
        let a = area_with_tiles(&[(0, 0)]);
        let b = area_with_tiles(&[(0, 1)]);
        assert!(a.is_adjacent(&b));
    }
}
