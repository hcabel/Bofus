use bevy::prelude::*;

#[derive(Component)]
pub struct CombatButton;

pub fn spawn_ready_button<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
    let mut button = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Px(100.0),
            height: Val::Px(50.0),
            ..default()
        },
        BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
        CombatButton,
    ));
    button.with_children(|parent| {
        parent.spawn((
            Text::new("Ready"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
        ));
    });
    let button_id = button.id();
    spawn_timer_bar(commands).set_parent(button_id);
    commands.entity(button_id)
}

pub fn spawn_end_turn_button<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
    let mut button = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Px(100.0),
            height: Val::Px(50.0),
            ..default()
        },
        BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
        CombatButton,
    ));
    button.with_children(|parent| {
        parent.spawn((
            Text::new("End turn"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
        ));
    });
    let button_id = button.id();
    spawn_timer_bar(commands).set_parent(button_id);
    commands.entity(button_id)
}

#[derive(Component)]
pub struct CombatTimerBar;

fn spawn_timer_bar<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
    let mut bar = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::hsl(244.0, 0.17, 0.19)),
    ));
    bar.with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::hsl(33.0, 0.90, 0.70)),
            CombatTimerBar,
        ));
    });
    bar
}
