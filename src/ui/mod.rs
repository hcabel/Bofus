use bevy::{prelude::*, reflect::MapInfo};
use context_menu::spawn;

use crate::{exploration, map, player, GameMode};

pub mod context_menu;
pub mod input_catcher;
pub mod popup;

pub fn init(app: &mut App) {}

fn setup(mut commands: Commands, player_info: Res<player::Info>) {
    let hud = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(90.0),
                height: Val::Percent(10.0),
                left: Val::Percent(17.5),
                width: Val::Percent(65.0),
                ..default()
            },
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
        ))
        .id();

    // Chat box
    commands
        .spawn((
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(40.0),
                ..default()
            },
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
        ))
        .set_parent(hud);

    // Player stats
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                height: Val::Percent(100.0),
                width: Val::Percent(10.0),
                ..default()
            },
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
        ))
        .set_parent(hud)
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(20.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BorderColor(Color::hsl(242.0, 0.15, 0.57)),
                ))
                .with_child((
                    Text::new(player_info.name.clone()),
                    TextColor(Color::hsl(242.0, 0.15, 0.57)),
                    TextLayout {
                        justify: JustifyText::Center,
                        linebreak: LineBreak::NoWrap,
                    },
                    TextFont {
                        font_size: 10.0,
                        ..Default::default()
                    },
                ));

            parent
                .spawn((Node {
                    display: Display::Flex,
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(100.0),
                    ..default()
                },))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                margin: UiRect::all(Val::Px(5.0)),
                                height: Val::Percent(100.0),
                                width: Val::Percent(60.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::hsl(360.0, 0.5, 0.5)),
                        ))
                        .with_child((
                            Text::new(player_info.max_health.to_string()),
                            TextColor(Color::hsl(190.0, 0.86, 0.97)),
                            TextLayout {
                                justify: JustifyText::Center,
                                linebreak: LineBreak::NoWrap,
                            },
                            TextFont {
                                font_size: 20.0,
                                ..Default::default()
                            },
                        ));
                    parent
                        .spawn((Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            height: Val::Percent(100.0),
                            width: Val::Percent(40.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    Node {
                                        margin: UiRect::all(Val::Px(5.0)),
                                        justify_content: JustifyContent::Center,
                                        height: Val::Percent(50.0),
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::hsl(209.0, 0.59, 0.61)),
                                ))
                                .with_child((
                                    Text::new(player_info.movement_points.to_string()),
                                    TextColor(Color::hsl(190.0, 0.86, 0.97)),
                                    TextLayout {
                                        justify: JustifyText::Center,
                                        linebreak: LineBreak::NoWrap,
                                    },
                                    TextFont {
                                        font_size: 20.0,
                                        ..Default::default()
                                    },
                                ));
                            parent
                                .spawn((
                                    Node {
                                        margin: UiRect::all(Val::Px(5.0)),
                                        justify_content: JustifyContent::Center,
                                        height: Val::Percent(50.0),
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::hsl(111.0, 0.52, 0.65)),
                                ))
                                .with_child((
                                    Text::new(player_info.movement_points.to_string()),
                                    TextColor(Color::hsl(190.0, 0.86, 0.97)),
                                    TextLayout {
                                        justify: JustifyText::Center,
                                        linebreak: LineBreak::NoWrap,
                                    },
                                    TextFont {
                                        font_size: 20.0,
                                        ..Default::default()
                                    },
                                ));
                        });
                });
        });

    // Player spells
    commands
        .spawn((
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(40.0),
                ..default()
            },
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
        ))
        .set_parent(hud);

    // Menus
    commands
        .spawn((
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(10.0),
                ..default()
            },
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
        ))
        .set_parent(hud);
}

#[derive(Component)]
pub struct MapUiInfo;

pub fn update_map_info<'a>(
    commands: &'a mut Commands,
    chunk_coordinate: map::ChunkCoordinate,
    q_map_info: Query<Entity, With<MapUiInfo>>,
) -> EntityCommands<'a> {
    if let Some(entity) = q_map_info.get_single().ok() {
        commands.entity(entity).despawn_recursive();
    }
    let root = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MapUiInfo,
        ))
        .id();
    let _region_name = commands
        .spawn((Node { ..default() }, Text::new("Incarnam")))
        .set_parent(root)
        .id();
    let _chunk_coordinate = commands
        .spawn((
            Node { ..default() },
            Text::new(format!("{}, {}", chunk_coordinate.x, chunk_coordinate.z)),
        ))
        .set_parent(root)
        .id();

    commands.entity(root)
}
