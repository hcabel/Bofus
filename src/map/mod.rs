use crate::materials::grid::GridMaterial;
use crate::player::MainPlayer;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_raycast::prelude::Raycast;

pub mod chunk;
pub mod tile;

#[cfg(debug_assertions)]
pub mod debug;

pub use chunk::Chunk;
pub use chunk::ChunkCoordinate;
pub use tile::Tile;
pub use tile::TileCoordinate;

pub fn init(app: &mut App) {
    app.init_asset::<Chunk>()
        .init_asset_loader::<chunk::loader::Loader>();
}

const MIN_CHUNK_X: i32 = -3;
const MIN_CHUNK_Y: i32 = -6;
const MAX_CHUNK_X: i32 = 5;
const MAX_CHUNK_Y: i32 = 1;

pub const ARRAY_OFFSET_X: i32 = -MIN_CHUNK_X;
pub const ARRAY_OFFSET_Y: i32 = -MIN_CHUNK_Y;

pub const SIZE_X: usize = (MAX_CHUNK_X - MIN_CHUNK_X + 1) as usize;
pub const SIZE_Z: usize = (MAX_CHUNK_Y - MIN_CHUNK_Y + 1) as usize;

#[derive(Component)]
pub struct MapComponent;

#[derive(Resource, Clone)]
pub struct CurrentChunk {
    pub grid: Handle<Chunk>,
    pub background: Handle<Image>,
}

#[derive(Resource)]
pub struct PreLoadedChunk {
    top: Option<CurrentChunk>,
    bottom: Option<CurrentChunk>,
    right: Option<CurrentChunk>,
    left: Option<CurrentChunk>,
}

#[derive(Resource, Default)]
pub struct Handles {
    pub ground_material: Handle<GridMaterial>,
    pub ground_mesh: Handle<Mesh>,
    pub block_material: Handle<StandardMaterial>,
    pub block_mesh: Handle<Mesh>,
    pub background_material: Handle<StandardMaterial>,
}

pub fn load_chunk(
    chunk_coordinates: ChunkCoordinate,
    mut commands: Commands,
    asset_server: &Res<AssetServer>,
) -> CurrentChunk {
    let current_chunk =
        chunk::load(chunk_coordinates, &asset_server).expect("Player shouldn't be out of bounds");
    let right_chunk = chunk::load(
        ChunkCoordinate::new(chunk_coordinates.x + 1, chunk_coordinates.z),
        &asset_server,
    );
    let left_chunk = chunk::load(
        ChunkCoordinate::new(chunk_coordinates.x - 1, chunk_coordinates.z),
        &asset_server,
    );
    let down_chunk = chunk::load(
        ChunkCoordinate::new(chunk_coordinates.x, chunk_coordinates.z + 1),
        &asset_server,
    );
    let bottom_chunk = chunk::load(
        ChunkCoordinate::new(chunk_coordinates.x, chunk_coordinates.z - 1),
        &asset_server,
    );
    commands.insert_resource(current_chunk.clone());
    commands.insert_resource(PreLoadedChunk {
        top: down_chunk,
        bottom: bottom_chunk,
        right: right_chunk,
        left: left_chunk,
    });
    current_chunk
}

pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tile_materials: ResMut<Assets<GridMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Sky light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
    });
    // Sun light
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_euler(
                EulerRot::YXZ,
                std::f32::consts::FRAC_PI_4 * 3.0,
                -std::f32::consts::FRAC_PI_4,
                0.0,
            ),
            ..default()
        },
        bevy::pbr::CascadeShadowConfig::default(),
    ));
    // create map materials and meshes
    let half_tile_size = tile::SIZE / 2.0;
    commands.insert_resource(Handles {
        ground_material: tile_materials.add(GridMaterial {
            color: LinearRgba::new(0.1, 0.1, 0.1, 1.0),
            thickness: 0.05,
        }),
        ground_mesh: meshes.add(Plane3d::new(
            Vec3::Y,
            Vec2::new(half_tile_size, half_tile_size),
        )),
        block_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.25),
            ..default()
        }),
        block_mesh: meshes.add(Cuboid::new(tile::SIZE, 0.5, tile::SIZE)),
        ..default()
    });
}

#[cfg(debug_assertions)]
pub(super) fn tile_editor(
    mut commands: Commands,
    args: Res<crate::ProcessArgs>,
    button_input: Res<ButtonInput<MouseButton>>,
    mut raycast: Raycast,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut chunk_assets: ResMut<Assets<Chunk>>,
    current_chunk: Res<CurrentChunk>,
    map_handles: ResMut<Handles>,
    q_tile: Query<(Entity, &Transform), With<Tile>>,
    q_player: Query<(&Transform, &MainPlayer)>,
) {
    use bevy_mod_raycast::prelude::RaycastSettings;
    if !args.map_editor {
        return;
    }
    if button_input.pressed(MouseButton::Left)
        || button_input.pressed(MouseButton::Right)
        || button_input.pressed(MouseButton::Middle)
    {
        let (camera, camera_transform) = q_camera.single();
        let window = q_window.single();
        let Some(mouse_position) = window.cursor_position() else {
            return;
        };
        let world_mouse_ray = camera
            .viewport_to_world(camera_transform, mouse_position)
            .unwrap();
        let raytrace = raycast.cast_ray(world_mouse_ray, &RaycastSettings::default());
        let player_chunk = ChunkCoordinate::from_world(q_player.single().0.translation);
        let Some((_entity, interaction)) = raytrace.first() else {
            warn!("Can't edit tiles from other chunks");
            return;
        };

        let new_tile = if button_input.pressed(MouseButton::Middle) {
            Tile::Empty
        } else if button_input.pressed(MouseButton::Left) {
            Tile::Block
        } else {
            Tile::Ground
        };
        let tile_coord = TileCoordinate::from_world(interaction.position());
        let current_chunk = chunk_assets.get_mut(current_chunk.grid.id()).unwrap();
        let tile = current_chunk.get_tile_mut(tile_coord.to_local()).unwrap();
        if *tile == new_tile {
            return;
        }
        for (entity, transform) in q_tile.iter() {
            let tile_tile_coord = TileCoordinate::from_world(transform.translation);
            if tile_tile_coord == tile_coord {
                commands.entity(entity).despawn_recursive();
            }
        }
        tile::spawn(&mut commands, &map_handles, new_tile, tile_coord.to_world());
        *tile = new_tile;
        chunk::save(&current_chunk, player_chunk);
    }
}
