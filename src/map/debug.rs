use crate::{map, player::MainPlayer};
use bevy::prelude::*;

pub fn draw_player_tile_gizmo(player: Query<&Transform, With<MainPlayer>>, mut gizmos: Gizmos) {
    let player_transform = player.single();
    let tile_coord = map::TileCoordinate::from_world(player_transform.translation);
    gizmos.rect(
        Isometry3d::new(
            tile_coord.to_world() + Vec3::new(0.0, 0.01, 0.0),
            Quat::from_euler(
                EulerRot::YXZ,
                std::f32::consts::FRAC_PI_4,
                -std::f32::consts::FRAC_PI_2,
                0.0,
            ),
        ),
        Vec2::new(map::tile::SIZE, map::tile::SIZE),
        if tile_coord.on_odd_row() {
            Color::hsl(0.0, 1.0, 0.5)
        } else {
            Color::hsl(200.0, 1.0, 0.5)
        },
    );
}

pub fn draw_player_chunk_gizmo(player: Query<&Transform, With<MainPlayer>>, mut gizmos: Gizmos) {
    let player_transform = player.single();
    let chunk_coord = map::ChunkCoordinate::from_world(player_transform.translation);
    let center = chunk_coord.world_center();
    gizmos.sphere(
        Isometry3d::new(
            center + Vec3::new(0.0, 0.01, 0.0),
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        ),
        0.1,
        Color::hsl(150.0, 1.0, 0.5),
    );

    let center = chunk_coord.world_center();
    let sizes = map::ChunkCoordinate::world_sizes();
    gizmos.rect(
        Isometry3d::new(
            center + Vec3::new(0.0, 0.15, 0.0),
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        ),
        Vec2::new(sizes.x, sizes.z),
        Color::hsl(150.0, 1.0, 0.5),
    );
}
