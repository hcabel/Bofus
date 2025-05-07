use bevy::{
    core_pipeline::tonemapping::Tonemapping, prelude::*, render::view::RenderLayers,
    window::PrimaryWindow, winit::cursor::CursorIcon,
};
use bevy_mod_raycast::prelude::DeferredRaycastingPlugin;
use clap::Parser;

mod combat;
mod exploration;
// mod loading;
mod map;
mod materials;
mod player;
mod save;
mod socket;
mod ui;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameMode {
    #[default]
    Exploration,
    MainMenu,
    Combat,
}

#[derive(Resource, Parser, Debug)]
#[command(version, about, long_about = None)]
struct ProcessArgs {
    /// Name of the file to get the player info from
    #[cfg(debug_assertions)]
    #[arg(long)]
    player_info_file: Option<String>,
    /// Whether or not we want to open the map editor
    #[cfg(debug_assertions)]
    #[arg(long)]
    map_editor: bool,
    /// Draw map grid on top of background
    #[arg(long, short)]
    show_grid: bool,
}

fn main() {
    let args = ProcessArgs::parse();
    let use_grid = args.show_grid;

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        // loading::Plugin,
        DeferredRaycastingPlugin::<()>::default(),
    ))
    .insert_resource(args)
    .init_state::<GameMode>()
    .add_systems(Startup, spawn_main_camera)
    .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9)))
    .add_systems(
        PreUpdate,
        |mut commands: Commands, q_window: Query<Entity, With<PrimaryWindow>>| {
            for window in q_window.iter() {
                commands.entity(window).remove::<CursorIcon>();
            }
        },
    );
    #[cfg(debug_assertions)]
    app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin {
        config: bevy::dev_tools::fps_overlay::FpsOverlayConfig {
            enabled: false,
            ..default()
        },
    })
    .add_systems(Update, dev_interaction);

    materials::player_shadow::init(&mut app);
    if use_grid {
        materials::grid::init(&mut app);
    }

    ui::init(&mut app);
    exploration::init(&mut app);
    combat::init(&mut app);
    socket::init(&mut app);

    app.run();
}

#[derive(Component)]
struct MainCamera;

fn spawn_main_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 5,
            ..default()
        },
        RenderLayers::layer(1),
        Tonemapping::None,
        MainCamera,
    ));
}

#[cfg(debug_assertions)]
fn dev_interaction(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_mode: Res<State<GameMode>>,
    mut next_gamemode: ResMut<NextState<GameMode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        match game_mode.get() {
            GameMode::Exploration => next_gamemode.set(GameMode::Combat),
            GameMode::Combat => next_gamemode.set(GameMode::Exploration),
            _ => {}
        }
    }
}
