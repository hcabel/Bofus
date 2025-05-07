use bevy::prelude::*;

#[derive(Component)]
pub struct ContextMenuTarget;

pub fn spawn<'a>(
    commands: &'a mut Commands,
    position: Vec2,
    create_button: impl FnOnce(&'a mut Commands, Entity),
) {
    info!("Creating player context menu");
    let input_catcher = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ZIndex(100),
        ))
        .observe(
            |mut trigger: Trigger<Pointer<Down>>, mut commands: Commands| {
                trigger.propagate(false);
                commands.entity(trigger.entity()).despawn_recursive();
            },
        )
        .id();

    let content = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                top: Val::Px(position.y),
                left: Val::Px(position.x),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(5.0)),
            BackgroundColor(Color::hsl(246.0, 0.21, 0.29)),
            ZIndex(101),
        ))
        .set_parent(input_catcher)
        .id();
    create_button(commands, content);
}
