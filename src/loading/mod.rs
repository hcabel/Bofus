// use std::any::Any;

// use bevy::prelude::{Plugin as BevyPlugin, *};

// #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
// pub enum LoadingState {
//     /// Waiting to be triggered
//     #[default]
//     Idle,
//     /// Is waiting for all the assets to be loaded
//     Loading,
//     /// Has just completed the loading process
//     Completed,
// }

// pub struct Plugin;

// impl BevyPlugin for Plugin {
//     fn build(&self, app: &mut App) {
//         app.init_state::<LoadingState>()
//             .add_systems(Update, (poll.run_if(in_state(LoadingState::Loading)),));
//     }
// }

// #[derive(Component)]
// pub struct Waiter(pub UntypedHandle);

// #[derive(Component)]
// pub struct Callback(pub Box<dyn Any + Send + Sync + 'static>);

// impl Callback {
//     pub fn new<I: SystemInput, Out, Marker, S: IntoSystem<I, Out, Marker>>(system: S) -> Self {
//         Self(Box::new(IntoSystem::into_system(system)))
//     }
// }

// pub fn start(mut next_state: ResMut<NextState<LoadingState>>, state: Res<State<LoadingState>>) {
//     next_state.set(LoadingState::Loading);
//     if *state.get() == LoadingState::Idle {
//         next_state.set(LoadingState::Loading);
//     }
// }

// pub fn poll(
//     mut commands: Commands,
//     q_waiters: Query<(Entity, &Waiter, Option<&Callback>)>,
//     mut next_state: ResMut<NextState<LoadingState>>,
//     asset_server: Res<AssetServer>,
// ) {
//     let mut count = 0;
//     for (entity, waiter, callback) in q_waiters.iter() {
//         count += 1;
//         if asset_server
//             .get_load_state(waiter.0.id())
//             .unwrap()
//             .is_loaded()
//             == false
//         {
//             continue;
//         }
//         if let Some(callback) = callback {

//         }
//         commands.entity(entity).despawn_recursive();
//     }
//     if count == 0 {
//         next_state.set(LoadingState::Completed);
//         return;
//     }
// }
