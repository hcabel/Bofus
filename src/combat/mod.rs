use crate::{
    map::{self, tile::LocalSpace, ChunkCoordinate},
    player::{CameraPivot, MainPlayer, Player, PLAYER_SIZES},
    GameMode,
};
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;

mod preparation;
mod ui;
// mod your_turn;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum CombatState {
    #[default]
    NotInCombat,
    Preparation,
    NextTurn,
    YourTurn,
    OthersTurn,
    End,
}

#[derive(Resource)]
struct CombatTimer(pub Timer);

#[derive(Resource)]
struct TurnSequence(Vec<Player>);

#[derive(Resource)]
pub struct Owner(pub PeerId);

pub fn init(app: &mut App) {
    preparation::init(app);
    //     your_turn::init(app);
    app.add_systems(
        OnTransition {
            exited: GameMode::Exploration,
            entered: GameMode::Combat,
        },
        combat_setup,
    )
    .init_state::<CombatState>();
    //     .add_systems(OnExit(GameMode::Combat), cleanup)
    //     .add_systems(OnEnter(CombatState::NextTurn), determine_whose_turn);
}

fn combat_setup(
    mut commands: Commands,
    // q_map_entities: Query<Entity, With<world::map::SceneComponent>>,
    q_player: Query<&Transform, (With<MainPlayer>, Without<CameraPivot>)>,
    chunk_assets: Res<Assets<map::Chunk>>,
    current_chunk: Res<map::CurrentChunk>,
    map_handles: ResMut<map::Handles>,
    mut q_camera: Query<&mut Transform, (With<CameraPivot>, Without<MainPlayer>)>,
    mut next_combat_state: ResMut<NextState<CombatState>>,
) {
    info!("Entering combat");
    let player_transform = q_player.single();
    let chunk_index = ChunkCoordinate::from_world(player_transform.translation);
    let chunk = chunk_assets.get(current_chunk.grid.id()).unwrap();
    for z in 0..map::chunk::SIZE_Z {
        for x in 0..map::chunk::SIZE_X {
            let tile = chunk.0[z][x];
            let tile_coord =
                map::TileCoordinate::<LocalSpace>::new(x as i32, z as i32).to_absolute(chunk_index);
            map::tile::spawn(&mut commands, &map_handles, tile, tile_coord.to_world());
        }
    }
    let mut camera = q_camera.single_mut();
    let chunk_center = chunk_index.world_center();
    camera.translation = chunk_center;
    next_combat_state.set(CombatState::Preparation);
}

// fn determine_whose_turn(
//     mut next_combat_state: ResMut<NextState<CombatState>>,
//     combat_state: Res<State<CombatState>>,
// ) {
//     // TMP logi
//     match combat_state.get() {
//         CombatState::YourTurn => next_combat_state.set(CombatState::OthersTurn),
//         CombatState::OthersTurn => next_combat_state.set(CombatState::YourTurn),
//         _ => next_combat_state.set(CombatState::YourTurn),
//     }
// }

// fn cleanup(
//     commands: Commands,
//     q_map_entities: Query<Entity, With<world::map::SceneComponent>>,
//     mut next_state: ResMut<NextState<CombatState>>,
// ) {
//     info!("Leaving combat");
//     despawn_map(commands, q_map_entities);
//     next_state.set(CombatState::NotInCombat);
// }

// fn despawn_map(
//     mut commands: Commands,
//     q_map_entities: Query<Entity, With<world::map::SceneComponent>>,
// ) {
//     for entity in q_map_entities.iter() {
//         commands.entity(entity).despawn_recursive();
//     }
// }
