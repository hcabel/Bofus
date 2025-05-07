use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::{
        camera::{RenderTarget, ScalingMode},
        render_resource::{TextureFormat, TextureUsages},
        view::{GpuCulling, RenderLayers},
    },
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};
use bevy_mod_raycast::prelude::{RaycastMesh, RaycastSource};
use loading::LoadingState;

use crate::{
    combat, map,
    player::{self, CameraPivot, MainPlayer, Player},
    save, socket, ui, GameMode,
};

mod loading;
// mod network;
// mod path_finding;

/// State only relevent if GameMode is Exploration
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    #[default]
    Loading,
    Exploration,
    Combat,
}

pub fn init(app: &mut App) {
    // network::init(app);
    map::init(app);
    loading::init(app);

    app.add_systems(OnEnter(GameMode::Exploration), loading::start)
        .add_systems(OnEnter(LoadingState::Completed), go_in_explaration_mode)
        .add_systems(
            OnEnter(State::Exploration),
            (
                map::setup,
                //map::spawn_fresh,
                //map::preload_adjacents,
                player::spawn_main_character,
            ),
        );

    // .add_systems(OnEnter(State::Exploration),
    // .add_systems(OnExit(GameMode::Exploration), clea)

    // #[cfg(debug_assertions)]
    // {
    //     app.add_systems(
    //         Update,
    //         (
    //             map::debug::draw_player_tile_gizmo,
    //             map::debug::draw_player_chunk_gizmo,
    //         )
    //             .run_if(in_state(GameMode::Exploration)),
    //     );
    // }
}

fn go_in_explaration_mode(
    mut next_game_mode: ResMut<NextState<GameMode>>,
    mut next_loading_state: ResMut<NextState<LoadingState>>,
    handles: Res<loading::Handles>,
) {
    next_loading_state.set(LoadingState::Idle); // Reset loading state
    next_game_mode.set(GameMode::Exploration);
}

// #[derive(Component)]
// pub struct ExplorationObserver;

// #[derive(Component)]
// #[require(Transform)]
// struct Raycaster;

// #[derive(Resource)]
// struct RenderTargetImage(Handle<Image>);

// #[derive(Resource)]
// struct CurrentBackground(Handle<Image>);

fn spawn_camera(mut commands: Commands, mut image_assets: ResMut<Assets<Image>>) {
    info!("Spawn camera");
    let image_handle = {
        let mut image = Image::default();
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;
        image_assets.add(image)
    };
    commands.insert_resource(RenderTargetImage(image_handle.clone()));
    let camera_pivot = commands
        .spawn((
            CameraPivot,
            Transform {
                rotation: Quat::from_rotation_x(-30.0f32.to_radians()),
                ..default()
            },
            Visibility::Hidden,
        ))
        .id();

    commands
        .spawn((
            Camera3d::default(),
            Camera {
                clear_color: ClearColorConfig::None,
                order: 3,
                ..default()
            },
            RenderLayers::layer(0),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: (20.0 * map::tile::SPACING_Z) / 100.0 * 116.0,
                },
                area: Rect {
                    min: Vec2::new(0.0, 0.0),
                    max: Vec2::new(0.0, (20.0 * map::tile::SPACING_Z) / 100.0 * 116.0),
                },
                viewport_origin: Vec2::new(0.5, 0.555),
                ..OrthographicProjection::default_2d()
            }),
            Tonemapping::None,
            Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            GpuCulling,
        ))
        .set_parent(camera_pivot);
}

// fn setup(
//     mut commands: Commands,
//     save_data_assets: Res<Assets<save::Data>>,
//     save_data_handle: Res<save::ResHandle>,
// ) {
//     info!("Setup");
//     // Spawn raycaster
//     commands.spawn((Raycaster, RaycastSource::<()>::new_transform_empty()));
//     // Trigger first chunk change
//     commands.spawn((Observer::new(on_chunk_transition), ExplorationObserver));
//     let save_data = save_data_assets.get(&save_data_handle.0).unwrap();
//     commands.trigger(ChunkChange(map::ChunkCoordinate::from_world(
//         save_data.player_position,
//     )));
// }

// fn move_raycaster_to_match_cursor(
//     mut q_raycaster: Query<&mut Transform, With<Raycaster>>,
//     q_camera: Query<(&Camera, &GlobalTransform)>,
//     q_window: Query<&Window, With<PrimaryWindow>>,
// ) {
//     // let (camera, camera_transform) = q_camera.single();
//     // let window = q_window.single();
//     // let Some(mouse_position) = window.cursor_position() else {
//     //     return;
//     // };
//     // let Ok(ray) = camera.viewport_to_world(camera_transform, mouse_position) else {
//     //     error!("Raycaster: Failed to compute mouse 3d ray");
//     //     return;
//     // };
//     // let mut raycaster_transform = q_raycaster.single_mut();
//     // raycaster_transform.translation = ray.origin;
//     // raycaster_transform.look_at(ray.origin + ray.direction.as_vec3(), Vec3::Y);
// }

// fn player_movement(
//     mut commands: Commands,
//     q_raycast_source: Query<&RaycastSource<()>>,
//     q_window: Query<Entity, With<PrimaryWindow>>,
//     chunk_assets: Res<Assets<map::Chunk>>,
//     current_chunk: Res<map::CurrentChunk>,
//     #[cfg(debug_assertions)] mut gizmos: Gizmos,
//     button_input: Res<ButtonInput<MouseButton>>,
//     q_player: Query<&Transform, (With<MainPlayer>, Without<CameraPivot>)>,
// ) {
//     let Some(intersections) = q_raycast_source.single().get_intersections() else {
//         return;
//     };
//     let Some(first_intersection) = intersections.first() else {
//         return;
//     };
//     let hit_tile_coord = map::TileCoordinate::from_world(first_intersection.1.position());
//     let chunk = chunk_assets.get(current_chunk.grid.id()).unwrap();
//     let Some(hit_tile) = chunk.get_tile(hit_tile_coord.to_local()) else {
//         return;
//     };
//     if *hit_tile != map::Tile::Ground {
//         return;
//     }
//     gizmos.rect(
//         Isometry3d::new(
//             hit_tile_coord.to_world() + Vec3::new(0.0, 0.01, 0.0),
//             Quat::from_euler(
//                 EulerRot::YXZ,
//                 std::f32::consts::FRAC_PI_4,
//                 -std::f32::consts::FRAC_PI_2,
//                 0.0,
//             ),
//         ),
//         Vec2::new(map::tile::SIZE, map::tile::SIZE),
//         if hit_tile_coord.on_odd_row() {
//             Color::hsl(0.0, 1.0, 0.5)
//         } else {
//             Color::hsl(200.0, 1.0, 0.5)
//         },
//     );
//     commands
//         .entity(q_window.single())
//         .insert(CursorIcon::System(SystemCursorIcon::Pointer));
//     if button_input.just_pressed(MouseButton::Left) {
//         let start = map::TileCoordinate::from_world(q_player.single().translation);
//         path_finding::spawn_path_finding_task(
//             commands,
//             start,
//             hit_tile_coord,
//             chunk_assets,
//             current_chunk,
//         );
//     }
// }

// #[derive(Event)]
// pub struct ChunkChange(pub map::ChunkCoordinate);

// #[derive(Resource)]
// #[allow(dead_code)]
// struct PreloadedBackgrounds([Handle<Image>; 4]);

// fn on_chunk_transition(
//     trigger: Trigger<ChunkChange>,
//     mut commands: Commands,
//     mut q_camera_pivot: Query<&mut Transform, With<CameraPivot>>,
//     q_projection: Query<&Projection>,
//     q_map_info: Query<Entity, With<ui::MapUiInfo>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     chunk_assets: Res<Assets<map::Chunk>>,
//     mut map_handles: ResMut<map::Handles>,
//     q_map_component: Query<Entity, With<map::MapComponent>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     asset_server: ResMut<AssetServer>,
//     args: Res<crate::ProcessArgs>,
//     render_target_image: Res<RenderTargetImage>,
// ) {
//     info!("Entering new chunk: {:?}", trigger.0);
//     let current_chunk = map::load_chunk(trigger.0, commands.reborrow(), &asset_server);
//     q_camera_pivot.single_mut().translation = trigger.0.world_center();
//     ui::update_map_info(&mut commands, trigger.0, q_map_info);
//     // despawn old map
//     for tile_entity in q_map_component.iter() {
//         commands.entity(tile_entity).despawn_recursive();
//     }
//     // spawn new tiles
//     if args.show_grid {
//         let chunk = chunk_assets.get(current_chunk.grid.id()).unwrap();
//         for z in 0..map::chunk::SIZE_Z {
//             for x in 0..map::chunk::SIZE_X {
//                 let tile_coord = map::TileCoordinate::new(
//                     trigger.0.x * map::chunk::SIZE_X as i32 + x as i32,
//                     trigger.0.z * map::chunk::SIZE_Z as i32 + z as i32,
//                 );
//                 let tile = chunk.get_tile(tile_coord.to_local()).unwrap().clone();
//                 map::tile::spawn(&mut commands, &map_handles, tile, tile_coord.to_world());
//             }
//         }
//     }
//     // // load new background texture
//     // map_handles.background_material = materials.add(StandardMaterial {
//     //     base_color_texture: Some(current_chunk.background),
//     //     depth_bias: -10.0, // FIXME: No effect
//     //     unlit: true,
//     //     ..default()
//     // });
//     // // spawn background
//     // const SCREEN_SIZE_X: u32 = 2636;
//     // const SCREEN_SIZE_Y: u32 = 1362;
//     // let Projection::Orthographic(orthographic) = q_projection.single() else {
//     //     panic!("Failed to get orthographic projection");
//     // };
//     // let height = orthographic.area.height();
//     // let ratio = height / SCREEN_SIZE_Y as f32;
//     // let width = SCREEN_SIZE_X as f32 * ratio / 2.0;
//     // let center = trigger.0.world_center() + Vec3::Z * height * 0.055 * 2.0
//     //     - Vec3::X * map::tile::SPACING_X * 0.25
//     //     - Vec3::Y * 0.1;
//     // commands.spawn((
//     //     Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(width, height)))),
//     //     MeshMaterial3d(map_handles.background_material.clone()),
//     //     Transform {
//     //         translation: center,
//     //         ..default()
//     //     },
//     //     map::MapComponent,
//     //     RaycastMesh::<()>::default(),
//     // ));
//     commands.spawn((
//         ImageNode::new(current_chunk.background),
//         Node {
//             position_type: PositionType::Absolute,
//             top: Val::ZERO,
//             bottom: Val::ZERO,
//             left: Val::ZERO,
//             right: Val::ZERO,
//             ..default()
//         },
//         RenderLayers::layer(2),
//         map::MapComponent,
//     ));
// }

// #[derive(Component)]
// struct Tooltip;

// fn player_name_hover(
//     mut commands: Commands,
//     q_transform: Query<&Transform>,
//     q_camera: Query<(&Camera, &GlobalTransform)>,
//     q_player: Query<(Entity, &Player)>,
//     q_tooltip: Query<Entity, With<Tooltip>>,
//     q_raycast_source: Query<&RaycastSource<()>>,
// ) {
//     for tooltip in q_tooltip.iter() {
//         commands.entity(tooltip).despawn_recursive();
//     }
//     let Some(entities_under_cursor) = q_raycast_source.single().get_intersections() else {
//         return;
//     };
//     for (tile_entity, info) in q_player.iter() {
//         for res_entry in entities_under_cursor.iter() {
//             if tile_entity == res_entry.0 {
//                 let player_info = info;
//                 let player_transform = q_transform.get(tile_entity).unwrap();
//                 let (camera, global_transform) = q_camera.single();
//                 let screen_position = camera
//                     .world_to_viewport(global_transform, player_transform.translation)
//                     .unwrap();
//                 commands
//                     .spawn((
//                         Node {
//                             top: Val::Px(screen_position.y - 50.0),
//                             left: Val::Px(screen_position.x - 50.0),
//                             width: Val::Px(100.0),
//                             ..default()
//                         },
//                         BorderRadius::all(Val::Px(5.0)),
//                         BackgroundColor(Color::hsla(0.0, 0.0, 0.25, 0.5)),
//                         Tooltip,
//                     ))
//                     .with_child((
//                         Text::new(player_info.name.clone()),
//                         TextFont {
//                             font_size: 20.0,
//                             ..default()
//                         },
//                     ));
//                 return;
//             }
//         }
//     }
// }

// // TODO: Can be called when already opened
// fn context_menu(
//     mut commands: Commands,
//     mouse_button_input: Res<ButtonInput<MouseButton>>,
//     q_player: Query<&Transform, With<Player>>,
//     q_camera: Query<(&Camera, &GlobalTransform)>,
//     q_raycast_source: Query<&RaycastSource<()>>,
//     q_main_player: Query<Entity, With<MainPlayer>>,
// ) {
//     if !mouse_button_input.just_pressed(MouseButton::Right) {
//         return;
//     }
//     let Some(res) = q_raycast_source.single().get_intersections() else {
//         return;
//     };
//     let (camera, camera_transform) = q_camera.single();
//     for entry in res.iter() {
//         if let Some(_) = q_main_player.get(entry.0).ok() {
//             continue;
//         }
//         let Some(player) = q_player.get(entry.0).ok() else {
//             continue;
//         };
//         let player_entity = entry.0;
//         let viewport_position = camera
//             .world_to_viewport(camera_transform, player.translation)
//             .unwrap();
//         ui::context_menu::spawn(&mut commands, viewport_position, |commands, entity| {
//             commands.entity(entity).with_children(|parent| {
//                 parent.spawn(Text::new("Duel")).observe(
//                     move |_trigger: Trigger<Pointer<Down>>,
//                           mut commands: Commands,
//                           q_peer_id: Query<&socket::Id>| {
//                         info!("Sending duel demand");
//                         let peer_id = q_peer_id.get(player_entity).unwrap().0;
//                         commands.trigger(socket::SendMessageEvent::ToPeer(
//                             socket::Message::DuelDemand,
//                             peer_id.clone(),
//                         ));
//                         ui::popup::spawn(&mut commands, "Duel demand sent".to_string()).observe(
//                             move |trigger: Trigger<ui::popup::CloseEvent>,
//                                   mut commands: Commands| {
//                                 info!("Duel demand cancelled");
//                                 commands.trigger(socket::SendMessageEvent::ToPeer(
//                                     socket::Message::DuelCancelled,
//                                     peer_id,
//                                 ));
//                                 commands.entity(trigger.entity()).despawn_recursive();
//                             },
//                         );
//                     },
//                 );
//             });
//         });
//         return;
//     }
// }

// fn cleanup(
//     mut commands: Commands,
//     q_map_components: Query<Entity, With<map::MapComponent>>,
//     q_observer: Query<Entity, With<ExplorationObserver>>,
// ) {
//     info!("Ending");
//     for entity in q_map_components.iter() {
//         commands.entity(entity).despawn_recursive();
//     }
//     for entity in q_observer.iter() {
//         commands.entity(entity).despawn_recursive();
//     }
// }
