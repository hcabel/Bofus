use std::ops::{Add, Div, Mul, Sub};

use bevy::{
    asset::{Asset, AssetServer, Handle},
    math::Vec3,
    prelude::{Res, ResMut},
    reflect::Reflect,
};

use crate::map;

use super::CurrentChunk;
pub mod loader;

pub const SIZE_X: usize = 14;
pub const SIZE_Z: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoordinate {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_tile(tile: map::TileCoordinate<map::tile::AbsoluteSpace>) -> Self {
        let x = (tile.x as f32 / SIZE_X as f32).floor() as i32;
        let z = (tile.z as f32 / SIZE_Z as f32).floor() as i32;
        Self::new(x, z)
    }

    pub fn from_world(world_coordinate: Vec3) -> Self {
        // let x = (world_coordinate.x / map::chunk::SIZE_X as f32).floor() as i32;
        // let z = (world_coordinate.z / map::chunk::SIZE_Z as f32).floor() as i32;
        // Self::new(x, z)
        map::TileCoordinate::from_world(world_coordinate).to_chunk()
    }

    pub fn world_sizes() -> Vec3 {
        Vec3::new(
            SIZE_X as f32 * map::tile::SPACING_X,
            0.0,
            SIZE_Z as f32 * map::tile::SPACING_Z,
        )
    }

    pub fn start(self) -> Vec3 {
        Vec3::new(self.x as f32, 0.0, self.z as f32) * Self::world_sizes()
    }

    pub fn end(self) -> Vec3 {
        self.start() + Self::world_sizes()
    }

    pub fn world_center(self) -> Vec3 {
        (self.start() + self.end()) / 2.0
    }
}

impl Add for ChunkCoordinate {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.z + rhs.z)
    }
}

impl Sub for ChunkCoordinate {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.z - rhs.z)
    }
}

impl Mul for ChunkCoordinate {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.z * rhs.z)
    }
}

impl Mul<i32> for ChunkCoordinate {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.z * rhs)
    }
}

impl Div for ChunkCoordinate {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.z / rhs.z)
    }
}

impl Div<i32> for ChunkCoordinate {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.z / rhs)
    }
}

#[derive(Asset, Reflect, Debug, Clone, Copy)]
pub struct Chunk(pub [[map::Tile; SIZE_X]; SIZE_Z]);

impl Chunk {
    pub fn get_tile(&self, tile: map::TileCoordinate<map::tile::LocalSpace>) -> Option<&map::Tile> {
        let tile = self.0.get(tile.z as usize)?.get(tile.x as usize)?;
        Some(tile)
    }
    pub fn get_tile_mut(
        &mut self,
        tile: map::TileCoordinate<map::tile::LocalSpace>,
    ) -> Option<&mut map::Tile> {
        let tile = self.0.get_mut(tile.z as usize)?.get_mut(tile.x as usize)?;
        Some(tile)
    }
}

pub fn load(coord: ChunkCoordinate, asset_server: &Res<AssetServer>) -> Option<CurrentChunk> {
    let background_path = format!("map/{},{}.png", coord.x, coord.z);
    let background = asset_server.load(background_path);
    let grid_data_path = format!("map/{},{}_TileData.ron", coord.x, coord.z);
    let grid = asset_server.load(grid_data_path);
    Some(CurrentChunk { grid, background })
}

pub fn save(chunk: &Chunk, coord: ChunkCoordinate) {
    let path = format!("map/{},{}_TileData.ron", coord.x, coord.z);
    let file = std::fs::File::create(path).unwrap();
    ron::ser::to_writer(file, chunk).unwrap();
}
