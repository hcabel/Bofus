use bevy::{
    prelude::*,
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};
use bevy_mod_raycast::prelude::Raycast;

use crate::{
    player::{PlayerInfo, PossessedPlayer, PLAYER_SIZES},
    world::{
        map::{self, MapResource},
        Map,
    },
};

use super::{
    ui::{self, CombatButton, CombatTimerBar},
    CombatState, CombatTimer,
};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum TurnAction {
    #[default]
    NotInTurn,
    WaitingNextAction,
    Move,
    UseSpell,
    EndTurn,
}

pub(super) fn init(app: &mut App) {
    app.init_state::<TurnAction>()
        .add_systems(OnEnter(CombatState::YourTurn), setup)
        .add_systems(Update, timer_update.run_if(in_state(CombatState::YourTurn)))
        .add_systems(OnEnter(TurnAction::WaitingNextAction), spawn_movement_tiles)
        .add_systems(
            Update,
            movement_tile_interaction.run_if(in_state(TurnAction::WaitingNextAction)),
        )
        .add_systems(OnExit(TurnAction::WaitingNextAction), clean_movement_tiles)
        .add_systems(OnEnter(TurnAction::Move), play_move_animation)
        .add_systems(OnEnter(TurnAction::EndTurn), end_turn)
        .add_systems(OnExit(CombatState::YourTurn), cleanup);
}

fn setup(
    mut commands: Commands,
    player_info: Res<PlayerInfo>,
    mut next_turn_action: ResMut<NextState<TurnAction>>,
) {
    info!("Turn started");
    commands.insert_resource(CombatTimer(Timer::from_seconds(30.0, TimerMode::Once)));
    ui::spawn_end_turn_button(&mut commands).observe(
        |_trigger: Trigger<Pointer<Up>>, mut next_turn_action: ResMut<NextState<TurnAction>>| {
            info!("Ending turn");
            next_turn_action.set(TurnAction::EndTurn);
        },
    );
    commands.insert_resource(PlayerInfoLeft {
        action_points: player_info.action_points,
        movement_points: player_info.movement_points,
    });
    next_turn_action.set(TurnAction::WaitingNextAction);
}

#[derive(Resource)]
pub struct PlayerInfoLeft {
    pub action_points: u8,
    pub movement_points: u8,
}

#[derive(Component)]
struct MovementTile {
    cost: u8,
}

fn spawn_movement_tiles(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_player: Query<&Transform, With<PossessedPlayer>>,
    map_res: Res<MapResource>,
    maps: Res<Assets<Map>>,
    player_info: Res<PlayerInfoLeft>,
) {
    info!("Spawn movement tiles");
    let player = q_player.single();
    let map = maps.get(&map_res.handle).unwrap();
    let player_chunk_x = 0;
    let player_chunk_z = 0;
    let player_chunk = &map.0[player_chunk_z][player_chunk_x];
    let (player_tile_x, player_tile_z) = map::world_to_tile(player.translation);
    map::flood_fill(
        player_chunk_x,
        player_chunk_z,
        player_tile_x,
        player_tile_z,
        player_info.movement_points as usize,
        |x, z, depth| {
            if depth == 0 {
                true // Skip the player tile
            } else if player_chunk.tiles[z][x] != map::Tile::Ground {
                false // Dead end
            } else {
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::hsl(100.0, 0.59, 0.20),
                        ..default()
                    })),
                    Transform::from_translation(
                        map::tile_index_to_world(player_chunk_x, player_chunk_z, x, z)
                            + Vec3::new(0.0, 0.01, 0.0),
                    ),
                    MovementTile { cost: depth as u8 },
                ));
                true
            }
        },
    );
}

fn movement_tile_interaction(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut raycast: Raycast,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_window_entt: Query<Entity, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_tiles: Query<(Entity, &mut MeshMaterial3d<StandardMaterial>), With<MovementTile>>,
    q_cost: Query<&MovementTile>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_transform: Query<&Transform, Without<PossessedPlayer>>,
    mut q_player: Query<&mut Transform, With<PossessedPlayer>>,
    mut player_info: ResMut<PlayerInfoLeft>,
    mut next_turn_action: ResMut<NextState<TurnAction>>,
) {
    let entities_under_cursor = {
        let (camera, camera_transform) = q_camera.single();
        let Some(window) = q_window.get_single().ok() else {
            return;
        };
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        {
            raycast
                .cast_ray(world_position, &default())
                .iter()
                .map(|intersection| intersection.0)
                .collect::<Vec<Entity>>()
        } else {
            Vec::default()
        }
    };

    let mut position_tile_under_cursor = None;
    for (tile_entity, mut material) in q_tiles.iter_mut() {
        for res_entry in entities_under_cursor.iter() {
            let entity = res_entry;
            if tile_entity == *entity {
                position_tile_under_cursor = Some(tile_entity);
                materials.get_mut(&mut material.0).unwrap().base_color =
                    Color::hsl(100.0, 0.59, 0.60);
            } else {
                materials.get_mut(&mut material.0).unwrap().base_color =
                    Color::hsl(100.0, 0.59, 0.20);
            }
        }
    }

    if let Some(tile_entity) = position_tile_under_cursor {
        commands
            .entity(q_window_entt.single())
            .insert(CursorIcon::System(SystemCursorIcon::Pointer));
        if mouse_button_input.just_pressed(MouseButton::Left) {
            let mut player = q_player.single_mut();
            let tile_position = q_transform.get(tile_entity).unwrap().translation;
            player.translation = tile_position + Vec3::new(0.0, PLAYER_SIZES.y / 2.0, 0.0);
            player_info.movement_points -= q_cost.get(tile_entity).unwrap().cost;
            info!("Player moved to {:?}", tile_position);
            next_turn_action.set(TurnAction::Move);
        }
    } else {
        commands
            .entity(q_window_entt.single())
            .remove::<CursorIcon>();
    }
}

fn clean_movement_tiles(mut commands: Commands, q_tile: Query<Entity, With<MovementTile>>) {
    for entity in q_tile.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn play_move_animation(mut next_turn_action: ResMut<NextState<TurnAction>>) {
    // TODO
    info!("Playing move animation");
    next_turn_action.set(TurnAction::WaitingNextAction);
}

fn timer_update(
    mut q_timer_bar: Query<&mut Node, With<CombatTimerBar>>,
    time: Res<Time>,
    mut timer: ResMut<CombatTimer>,
    mut next_turn_action: ResMut<NextState<TurnAction>>,
) {
    timer.0.tick(time.delta());
    for mut node in q_timer_bar.iter_mut() {
        node.width = Val::Percent(
            timer.0.elapsed().as_secs_f32() / timer.0.duration().as_secs_f32() * 100.0,
        );
    }
    if timer.0.finished() {
        info!("Time's up!");
        next_turn_action.set(TurnAction::EndTurn);
    }
}

fn end_turn(
    mut commands: Commands,
    mut next_combat_state: ResMut<NextState<CombatState>>,
    mut next_turn_action: ResMut<NextState<TurnAction>>,
    mut timer: ResMut<CombatTimer>,
) {
    info!("Turn ended");
    next_combat_state.set(CombatState::NextTurn);
    next_turn_action.set(TurnAction::NotInTurn);
    commands.remove_resource::<CombatTimer>();
}

fn cleanup(mut commands: Commands, q_button: Query<Entity, With<CombatButton>>) {
    info!("Cleanup");
    for entity in q_button.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
