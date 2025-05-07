use crate::ui;
use bevy::prelude::*;

#[derive(Component)]
pub struct Popup;

#[derive(Event)]
pub struct CloseEvent;

pub fn spawn<'a>(commands: &'a mut Commands, text: String) -> EntityCommands<'a> {
    let input_catcher = ui::input_catcher::spawn_without_observer(commands)
        .observe(
            |mut trigger: Trigger<Pointer<Down>>, mut commands: Commands| {
                trigger.propagate(false);
                commands.trigger(CloseEvent);
                commands.entity(trigger.entity()).despawn_recursive();
            },
        )
        .id();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                top: Val::Percent(50.0),
                left: Val::Percent(50.0),
                width: Val::Px(200.0),
                height: Val::Px(100.0),
                ..default()
            },
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
            Popup,
        ))
        .observe(|mut trigger: Trigger<Pointer<Down>>| {
            trigger.propagate(false);
        })
        .set_parent(input_catcher)
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextColor(Color::hsl(242.0, 0.15, 0.57)),
                TextLayout {
                    justify: JustifyText::Center,
                    linebreak: LineBreak::WordBoundary,
                },
                TextFont {
                    font_size: 10.0,
                    ..Default::default()
                },
            ));
            parent.spawn((Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },));
        });
    commands.entity(input_catcher)
}

#[derive(Event)]
pub enum ButtonEvent {
    Primary,
    Secondary,
}

pub fn spawn_with_choices<'a>(
    commands: &'a mut Commands,
    heading_text: impl Into<String>,
    primary_button_text: impl Into<String>,
    secondary_button_text: impl Into<String>,
) -> EntityCommands<'a> {
    let input_catcher = ui::input_catcher::spawn_without_observer(commands)
        .observe(
            |mut trigger: Trigger<Pointer<Down>>, mut commands: Commands| {
                trigger.propagate(false);
                commands.trigger(ButtonEvent::Secondary);
                commands.entity(trigger.entity()).despawn_recursive();
            },
        )
        .id();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                top: Val::Percent(50.0),
                left: Val::Percent(50.0),
                width: Val::Px(200.0),
                height: Val::Px(100.0),
                ..default()
            },
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
            Popup,
        ))
        .set_parent(input_catcher)
        .observe(|mut trigger: Trigger<Pointer<Down>>| {
            trigger.propagate(false);
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new(heading_text.into()),
                TextColor(Color::hsl(242.0, 0.15, 0.57)),
                TextLayout {
                    justify: JustifyText::Center,
                    linebreak: LineBreak::WordBoundary,
                },
                TextFont {
                    font_size: 10.0,
                    ..Default::default()
                },
            ));
            parent
                .spawn((Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Text::new(primary_button_text.into()),
                            TextColor(Color::hsl(242.0, 0.15, 0.57)),
                            TextLayout {
                                justify: JustifyText::Center,
                                linebreak: LineBreak::WordBoundary,
                            },
                            TextFont {
                                font_size: 10.0,
                                ..Default::default()
                            },
                            BackgroundColor(Color::hsl(209.0, 0.59, 0.61)),
                        ))
                        .observe(
                            |_trigger: Trigger<Pointer<Down>>,
                             mut cmds: Commands,
                             q_popups: Query<Entity, With<Popup>>| {
                                cmds.trigger(ButtonEvent::Primary);
                                for popup in q_popups.iter() {
                                    cmds.entity(popup).despawn_recursive();
                                }
                            },
                        );
                    parent
                        .spawn((
                            Text::new(secondary_button_text.into()),
                            TextColor(Color::hsl(242.0, 0.15, 0.57)),
                            TextLayout {
                                justify: JustifyText::Center,
                                linebreak: LineBreak::WordBoundary,
                            },
                            TextFont {
                                font_size: 10.0,
                                ..Default::default()
                            },
                            BackgroundColor(Color::hsl(111.0, 0.52, 0.65)),
                        ))
                        .observe(
                            |_trigger: Trigger<Pointer<Down>>,
                             mut commands: Commands,
                             q_popups: Query<Entity, With<Popup>>| {
                                commands.trigger(ButtonEvent::Secondary);
                                for popup in q_popups.iter() {
                                    commands.entity(popup).despawn_recursive();
                                }
                            },
                        );
                });
        });
    commands.entity(input_catcher)
}
