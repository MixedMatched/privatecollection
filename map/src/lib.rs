use core::fmt;

use bevy::prelude::{Component, Quat};
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Copy, Clone, Debug, Component, PartialEq, Eq, PartialOrd, Ord, Default,
)]
pub enum TileType {
    Walkable,
    #[default]
    Blocked,
}

#[derive(
    Serialize, Deserialize, Copy, Clone, Debug, Component, PartialEq, Eq, PartialOrd, Ord, Default,
)]
pub enum ObjectType {
    #[default]
    Wall,
    Door,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Wall => write!(f, "Wall"),
            ObjectType::Door => write!(f, "Door"),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Component, PartialEq, Default)]
pub struct Object {
    pub object_type: ObjectType,
    pub rotation: Quat,
}

#[derive(
    Serialize, Deserialize, Copy, Clone, Debug, Component, PartialEq, Eq, PartialOrd, Ord, Default,
)]
pub struct FloorObject {
    pub object_type: ObjectType,
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Component, PartialEq, Eq, PartialOrd, Ord, Default,
)]
pub struct Connection {
    pub map: String,
    pub spawn: (usize, usize),
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map: {}, Spawn: {:?}", self.map, self.spawn)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Component, PartialEq, Default)]
pub struct Tile {
    pub tile_type: TileType,
    pub object: Option<Object>,
    pub floor_object: Option<FloorObject>,
    pub connection: Option<Connection>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Component, PartialEq, Default)]
pub struct Map {
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn expand_to(&mut self, y: i32, x: i32) {
        if (self.tiles.len() as i32) < x + 1 {
            self.tiles.resize(
                (x + 1).try_into().unwrap(),
                vec![Tile::default(); self.tiles.get(0).unwrap_or(&vec![]).len()],
            );
        }
        for row in self.tiles.iter_mut() {
            if (row.len() as i32) < y + 1 {
                row.resize((y + 1).try_into().unwrap(), Tile::default());
            }
        }

        if 0 > x + 1 {
            for _ in (x + 1)..0 {
                self.tiles.insert(
                    0,
                    vec![Tile::default(); self.tiles.get(0).unwrap_or(&vec![]).len()],
                )
            }
        }
        for row in self.tiles.iter_mut() {
            if 0 > y + 1 {
                for _ in (y + 1)..0 {
                    row.insert(0, Tile::default());
                }
            }
        }
    }

    pub fn trim(&mut self) {
        let horizontal_lower_bound = self
            .tiles
            .iter()
            .enumerate()
            .find(|(_, row)| row.iter().any(|tile| tile.tile_type != TileType::Blocked))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let horizontal_upper_bound = self
            .tiles
            .iter()
            .enumerate()
            .rev()
            .find(|(_, row)| row.iter().any(|tile| tile.tile_type != TileType::Blocked))
            .map(|(i, _)| i)
            .unwrap_or(self.tiles.len() - 1);

        let vertical_lower_bound = self
            .tiles
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .find(|(_, tile)| tile.tile_type != TileType::Blocked)
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            })
            .min()
            .unwrap_or(0);
        let vertical_upper_bound = self
            .tiles
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .rev()
                    .find(|(_, tile)| tile.tile_type != TileType::Blocked)
                    .map(|(i, _)| i)
                    .unwrap_or(row.len() - 1)
            })
            .max()
            .unwrap_or(self.tiles.get(0).map(|row| row.len() - 1).unwrap_or(0));

        self.tiles = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= horizontal_lower_bound && *i <= horizontal_upper_bound)
            .map(|(_, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(i, _)| *i >= vertical_lower_bound && *i <= vertical_upper_bound)
                    .map(|(_, tile)| tile.clone())
                    .collect()
            })
            .collect();
    }

    pub fn pad(&mut self, padding: usize) {
        let length = self.tiles.len();
        let width = self.tiles.get(0).map(|row| row.len()).unwrap_or(0);

        let mut new_tiles = vec![vec![Tile::default(); width + padding * 2]; length + padding * 2];

        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                new_tiles[i + padding][j + padding] = tile.clone();
            }
        }

        self.tiles = new_tiles;
    }
}
