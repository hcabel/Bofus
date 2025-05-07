use bevy::prelude::*;

use crate::{map, save};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingState {
    #[default]
    Idle,
    Loading,
    Completed,
}

pub fn init(app: &mut App) {
    app.init_state::<LoadingState>()
        .add_systems(Update, poll.run_if(in_state(LoadingState::Loading)))
        .add_systems(
            OnTransition {
                exited: LoadingState::Completed,
                entered: LoadingState::Idle,
            },
            cleanup,
        );
}

#[derive(Resource)]
pub struct Handles(pub Vec<UntypedHandle>);

pub fn start(
    mut next_loading_state: ResMut<NextState<LoadingState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(Handles(vec![asset_server
        .load::<save::Data>("player_info.ron")
        .untyped()]));

    next_loading_state.set(LoadingState::Loading);
}

pub fn poll(
    mut next_loading_state: ResMut<NextState<LoadingState>>,
    asset_server: Res<AssetServer>,
    mut handles: ResMut<Handles>,
    save_data_assets: Res<Assets<save::Data>>,
) {
    let count = handles.0.len();
    if count == 1 {
        let save_data_handle = handles.0.first().unwrap().clone().typed::<save::Data>();
        if asset_server
            .get_load_state(save_data_handle.id())
            .unwrap()
            .is_loaded()
        {
            let save_data = save_data_assets.get(save_data_handle.id()).unwrap();
            let player_chunk_coords = map::ChunkCoordinate::from_world(save_data.player_position);
            let chunk = map::chunk::load(player_chunk_coords, &asset_server).unwrap();
            handles
                .0
                .extend([chunk.grid.untyped(), chunk.background.untyped()]);
        }
    }

    for handle in handles.0.iter() {
        if asset_server
            .get_load_state(handle.id())
            .unwrap()
            .is_loaded()
            == false
        {
            return;
        }
    }

    next_loading_state.set(LoadingState::Completed);
}

pub fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Handles>();
}

// commands.spawn((
//     loading::Waiter(asset_server.load::<save::Data>("player_info.ron").untyped()),
//     loading::Callback(Box::new(|handle| {
//         println!("Loaded: {:?}", handle);
//     })),
// ));
// commands.insert_resource(
//     loading::Node::<save::Data>::new_single(
//         asset_server.load("player_info.ron")
//     ),
//         .then(loading::Node::new_single("player_sprite.png")),
//     ));
