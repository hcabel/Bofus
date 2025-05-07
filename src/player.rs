use bevy::{math::u16, prelude::*};
use bevy_mod_raycast::prelude::RaycastMesh;
use serde::{Deserialize, Serialize};

use crate::{materials::player_shadow::PlayerShadowMaterial, save};

#[derive(Resource, Component, Default, Deserialize, Serialize, Clone, Debug)]
pub struct Info {
    pub name: String,
    pub max_health: u32,
    pub action_points: u8,
    pub movement_points: u8,
    #[serde(skip)]
    pub stats: PlayerStats,
}

#[derive(Default, Clone, Debug)]
#[allow(dead_code)]
pub struct PlayerStats {
    pub vitality: u16,
    pub agility: u16,
    pub chance: u16,
    pub strength: u16,
    pub intelligence: u16,
    pub unspent_points: u16,
}

#[derive(Component)]
pub struct Player {
    pub name: String,
}

#[derive(Component)]
pub struct MainPlayer;

pub const PLAYER_SIZES: Vec3 = Vec3::new(0.5, 1.0, 0.5);

#[derive(Component)]
pub struct CameraPivot;

pub fn spawn_main_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_shadow_materials: ResMut<Assets<PlayerShadowMaterial>>,
    player_info: Res<Info>,
    save_data_assets: Res<Assets<save::Data>>,
    save_data_handle: Res<save::ResHandle>,
) {
    let save_data = save_data_assets.get(&save_data_handle.0).unwrap();
    spawn_player_character(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut player_shadow_materials,
        Player {
            name: player_info.name.clone(),
        },
        save_data.player_position,
    )
    .insert(MainPlayer);
}

pub fn spawn_player_character<'a>(
    commands: &'a mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_shadow_materials: &mut ResMut<Assets<PlayerShadowMaterial>>,
    player: Player,
    position: Vec3,
) -> EntityCommands<'a> {
    let root = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(PLAYER_SIZES.x, PLAYER_SIZES.y, PLAYER_SIZES.z))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_translation(Vec3::new(position.x, PLAYER_SIZES.y / 2.0, position.z)),
            player,
            RaycastMesh::<()>::default(),
        ))
        .id();

    commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::new(
                Vec3::Y,
                Vec2::new(PLAYER_SIZES.x * 2.0, PLAYER_SIZES.z * 2.0),
            ))),
            Transform::from_translation(Vec3::new(0.0, -(PLAYER_SIZES.y / 2.0), 0.0)),
            MeshMaterial3d(player_shadow_materials.add(PlayerShadowMaterial {})),
        ))
        .set_parent(root);

    commands.entity(root)
}
