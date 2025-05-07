use bevy::{
    prelude::*,
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};
use bevy_mod_raycast::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    combat::{self, CombatTimer},
    map,
    player::{MainPlayer, Player, PLAYER_SIZES},
    socket,
};

use super::{
    ui::{CombatButton, CombatTimerBar},
    CombatState,
};

pub fn init(app: &mut App) {
    app.add_systems(
        OnEnter(CombatState::Preparation),
        (spawn_network_observers, setup),
    )
    .add_systems(
        Update,
        (update_timer, placement_tile_interaction).run_if(in_state(CombatState::Preparation)),
    )
    .add_systems(OnExit(CombatState::Preparation), cleanup);
}

#[derive(Component)]
struct PreparationObserver;

fn spawn_network_observers(mut commands: Commands) {
    info!("Spawning network observers");
    commands.spawn((Observer::new(on_peer_position_update), PreparationObserver));
    commands.spawn((
        Observer::new(on_combat_ready_state_changed),
        PreparationObserver,
    ));
    commands.spawn((Observer::new(on_combat_started), PreparationObserver));
    commands.spawn((Observer::new(on_player_join), PreparationObserver));
}

fn on_player_join(
    trigger: Trigger<socket::CombatPlayerJoinedEvent>,
    mut commands: Commands,
    mut q_players: Query<(&mut Transform, &socket::Id, Entity), With<Player>>,
) {
    info!("Player {} joined", trigger.peer_id);
    for (mut player_transform, player_id, player_entity) in q_players.iter_mut() {
        if player_id.0 == trigger.peer_id {
            player_transform.translation =
                Vec3::new(trigger.position.x, PLAYER_SIZES.y / 2.0, trigger.position.z);
            commands.entity(player_entity).insert(trigger.stats.clone());
            break;
        }
    }
}

fn on_combat_started(
    _trigger: Trigger<socket::CombatStartedEvent>,
    mut next_state: ResMut<NextState<CombatState>>,
) {
    info!("Combat started");
    next_state.set(CombatState::NextTurn);
}

fn on_peer_position_update(
    trigger: Trigger<socket::UpdatePlayerPositionEvent>,
    mut commands: Commands,
    q_players: Query<(Entity, &socket::Id), With<Player>>,
) {
    for player in q_players.iter() {
        if player.1 .0 == trigger.1 {
            info!("Player {} moved to {:?}", player.1 .0, trigger.0);
            commands
                .entity(player.0)
                .insert(Transform::from_translation(Vec3::new(
                    trigger.0.x,
                    PLAYER_SIZES.y / 2.0,
                    trigger.0.z,
                )));
            break;
        }
    }
}

#[derive(Component)]
struct PlayerReady;

fn on_combat_ready_state_changed(
    trigger: Trigger<socket::CombatReadyStateChangedEvent>,
    mut commands: Commands,
    q_players: Query<(Entity, &socket::Id), With<Player>>,
    q_player_ready: Query<&PlayerReady>,
    owner: Res<combat::Owner>,
    my_id: Res<socket::MyId>,
    mut next_state: ResMut<NextState<CombatState>>,
) {
    let mut player_count = 0;
    let mut ready_count = 0;
    for player in q_players.iter() {
        player_count += 1;
        if player.1 .0 == trigger.peer_id {
            info!("Player {} is ready: {}", player.1 .0, trigger.is_ready);
            if trigger.is_ready {
                ready_count += 1;
                commands.entity(player.0).insert(PlayerReady);
            } else {
                commands.entity(player.0).remove::<PlayerReady>();
            }
        } else {
            let ready = q_player_ready.get(player.0).is_ok();
            if ready {
                ready_count += 1;
            }
        }
    }
    if owner.0 == my_id.0 && player_count == ready_count {
        info!("All players are ready");
        commands.trigger(socket::SendMessageEvent::Broadcast(
            socket::Message::CombatStart,
        ));
        info!("Combat started");
        next_state.set(CombatState::NextTurn);
    }
}

#[derive(Component)]
struct PlacementTile;

fn setup(
    mut commands: Commands,
    q_ground_tiles: Query<(Entity, &map::Tile), With<map::Tile>>,
    mut q_player: Query<&mut Transform, With<MainPlayer>>,
    q_transform: Query<&Transform, Without<MainPlayer>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_handles: Res<map::Handles>,
) {
    info!("Setup");
    commands.insert_resource(CombatTimer(Timer::from_seconds(90.0, TimerMode::Once)));
    let mut rng = rand::thread_rng();
    let random_tiles = {
        let mut tiles = q_ground_tiles
            .iter()
            .filter_map(|(entity, tile_type)| {
                if *tile_type == map::Tile::Ground {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        tiles.shuffle(&mut rng);
        tiles.truncate(10);
        for tile in tiles.iter() {
            let transform = q_transform.get(*tile).unwrap();
            let position = transform.translation + Vec3::new(0.0, 0.01, 0.0);
            commands
                .spawn((
                    Mesh3d(map_handles.ground_mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.0, 0.3),
                        ..default()
                    })),
                    transform.with_translation(position),
                ))
                .insert(PlacementTile);
        }
        tiles
    };
    // Teleport player to one of the random tiles
    let mut player = q_player.single_mut();
    player.translation = q_transform.get(random_tiles[0]).unwrap().translation
        + Vec3::new(0.0, PLAYER_SIZES.y / 2.0, 0.0);
    combat::ui::spawn_ready_button(&mut commands).observe(
        |_trigger: Trigger<Pointer<Up>>, mut commands: Commands| {
            info!("You're ready!");
            commands.trigger(socket::SendMessageEvent::Broadcast(
                socket::Message::CombatReadyStateChanged(true),
            ));
        },
    );
    commands.trigger(socket::SendMessageEvent::Broadcast(
        socket::Message::UpdatePlayerPosition(socket::UpdatePlayerPosition {
            x: player.translation.x,
            z: player.translation.z,
        }),
    ));
}

fn update_timer(
    mut commands: Commands,
    mut q_timer_bar: Query<&mut Node, With<CombatTimerBar>>,
    time: Res<Time>,
    mut timer: ResMut<CombatTimer>,
    mut next_state: ResMut<NextState<CombatState>>,
    owner: Res<combat::Owner>,
    my_id: Res<socket::MyId>,
) {
    for mut node in q_timer_bar.iter_mut() {
        node.width = Val::Percent(
            timer.0.elapsed().as_secs_f32() / timer.0.duration().as_secs_f32() * 100.0,
        );
    }
    timer.0.tick(time.delta());
    if timer.0.finished() {
        info!("Time's up!");
        if owner.0 == my_id.0 {
            commands.trigger(socket::SendMessageEvent::Broadcast(
                socket::Message::CombatStart,
            ));
            info!("Combat started");
            next_state.set(CombatState::NextTurn);
        }
    }
}

fn placement_tile_interaction(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut raycast: Raycast,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_window_entt: Query<Entity, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_tiles: Query<(Entity, &mut MeshMaterial3d<StandardMaterial>), With<PlacementTile>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_transform: Query<&Transform, Without<MainPlayer>>,
    mut q_player: Query<&mut Transform, With<MainPlayer>>,
) {
    let entities_under_cursor = {
        let (camera, camera_transform) = q_camera.single();
        let window = q_window.single();
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
                materials.get_mut(&mut material.0).unwrap().base_color = Color::hsl(0.0, 1.0, 0.7);
            } else {
                materials.get_mut(&mut material.0).unwrap().base_color = Color::hsl(0.0, 1.0, 0.3);
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
            info!("Player moved to {:?}", tile_position);
            commands.trigger(socket::SendMessageEvent::Broadcast(
                socket::Message::UpdatePlayerPosition(socket::UpdatePlayerPosition {
                    x: tile_position.x,
                    z: tile_position.z,
                }),
            ));
        }
    } else {
        commands
            .entity(q_window_entt.single())
            .remove::<CursorIcon>();
    }
}

fn cleanup(
    mut commands: Commands,
    mut q_tiles: Query<Entity, With<PlacementTile>>,
    q_combat_button: Query<Entity, With<CombatButton>>,
    q_observers: Query<Entity, With<PreparationObserver>>,
) {
    info!("Cleaning up");
    commands.remove_resource::<CombatTimer>();
    for entity in q_tiles.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in q_combat_button.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in q_observers.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
