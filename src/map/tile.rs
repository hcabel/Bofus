use core::f32;
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use bevy::{pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_raycast::prelude::RaycastMesh;
use serde::{Deserialize, Serialize};

use crate::map;

/// # Space between tiles (in the x axis)
/// Different from the tile diameter because the tiles are rotated 45 degrees
pub const SPACING_X: f32 = 2.0;
/// # Space between tiles (in the z axis)
/// Different from the tile diameter because the tiles are rotated 45 degrees
pub const SPACING_Z: f32 = SPACING_X / 2.0;
/// # Tile mesh diameter
pub const SIZE: f32 = SPACING_X / std::f32::consts::SQRT_2;

/// Relative to the chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSpace;
// Relative to the world
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbsoluteSpace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCoordinate<State = AbsoluteSpace> {
    pub x: i32,
    pub z: i32,
    state: std::marker::PhantomData<State>,
}

impl Display for TileCoordinate<LocalSpace> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TileCoordinate<Local>({}, {})", self.x, self.z)
    }
}

impl Display for TileCoordinate<AbsoluteSpace> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TileCoordinate<Absolute>({}, {})", self.x, self.z)
    }
}

impl<State> TileCoordinate<State> {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            state: std::marker::PhantomData,
        }
    }
}

impl TileCoordinate<AbsoluteSpace> {
    pub fn from_world(world_position: Vec3) -> Self {
        let z = (world_position.z / SPACING_X * 2.0).round() as i32;
        let x = ((world_position.x - (z % 2) as f32 * SPACING_X / 2.0) / SPACING_X).round() as i32;
        Self::new(x, z)
    }

    pub fn to_world(self) -> Vec3 {
        let x = self.x as f32 * SPACING_X + (self.z % 2) as f32 * (SPACING_X / 2.0);
        let z = self.z as f32 * SPACING_Z;
        Vec3::new(x, 0.0, z)
    }

    pub fn to_chunk(self) -> map::ChunkCoordinate {
        map::ChunkCoordinate::from_tile(self)
    }

    pub fn to_local(self) -> TileCoordinate<LocalSpace> {
        let local_x = self.x.abs() % map::chunk::SIZE_X as i32;
        let local_z = self.z.abs() % map::chunk::SIZE_Z as i32;
        debug_assert!(
            local_x >= 0
                && local_x < map::chunk::SIZE_X as i32
                && local_z >= 0
                && local_z < map::chunk::SIZE_Z as i32,
            "Out of bounds tile coordinate: from ({}, {}) to ({}, {})",
            self.x,
            self.z,
            local_x,
            local_z
        );
        TileCoordinate::new(local_x, local_z)
    }

    pub fn on_odd_row(self) -> bool {
        self.z % 2 == 1
    }
}

impl TileCoordinate<LocalSpace> {
    pub fn to_absolute(self, chunk: map::ChunkCoordinate) -> TileCoordinate<AbsoluteSpace> {
        let absolute_x = chunk.x * map::chunk::SIZE_X as i32 + self.x;
        let absolute_z = chunk.z * map::chunk::SIZE_Z as i32 + self.z;
        TileCoordinate::new(absolute_x, absolute_z)
    }
}

impl Add for TileCoordinate {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.z + rhs.z)
    }
}

impl Sub for TileCoordinate {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.z - rhs.z)
    }
}

impl Mul for TileCoordinate {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.z * rhs.z)
    }
}

impl Mul<i32> for TileCoordinate {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.z * rhs)
    }
}

impl Div for TileCoordinate {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.z / rhs.z)
    }
}

impl Div<i32> for TileCoordinate {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.z / rhs)
    }
}

#[derive(
    Component, Reflect, Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq,
)]
pub enum Tile {
    Block,
    #[default]
    Ground,
    Empty,
}

pub fn spawn(
    commands: &mut Commands,
    map_handles: &ResMut<map::Handles>,
    tile: Tile,
    coordinate: Vec3,
) {
    match tile {
        Tile::Ground => {
            commands.spawn((
                Mesh3d(map_handles.ground_mesh.clone()),
                MeshMaterial3d(map_handles.ground_material.clone()),
                Transform {
                    translation: coordinate,
                    rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4),
                    scale: Vec3::new(1.025, 1.025, 1.025),
                },
                RenderLayers::layer(0),
                Tile::Ground,
                map::MapComponent,
                NotShadowCaster,
            ));
        }
        Tile::Block => {
            commands.spawn((
                Mesh3d(map_handles.block_mesh.clone()),
                MeshMaterial3d(map_handles.block_material.clone()),
                Transform {
                    translation: coordinate + Vec3::new(0.0, 0.25, 0.0),
                    rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4),
                    scale: Vec3::ONE,
                },
                RenderLayers::layer(0),
                Tile::Block,
                map::MapComponent,
            ));
        }
        Tile::Empty => {}
    }
}
