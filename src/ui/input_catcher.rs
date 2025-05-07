use bevy::prelude::*;

pub fn spawn<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
    let mut input_catcher = spawn_without_observer(commands);
    input_catcher.observe(
        |mut trigger: Trigger<Pointer<Down>>, mut commands: Commands| {
            trigger.propagate(false);
            commands.entity(trigger.entity()).despawn_recursive();
        },
    );
    input_catcher
}

pub fn spawn_without_observer<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ZIndex(100),
    ))
}
