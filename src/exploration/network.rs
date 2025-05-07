use crate::{exploration::*, socket};
use bevy::prelude::*;
use bevy_matchbox::prelude::PeerId;

pub fn init(app: &mut App) {
    app.add_systems(OnEnter(GameMode::Exploration), setup)
        .add_systems(
            Update,
            replicate_player_position
                .run_if(in_state(GameMode::Exploration))
                .run_if(socket::is_connected),
        );
}

fn setup(mut commands: Commands) {
    commands.spawn((Observer::new(on_peer_position_update), ExplorationObserver));
    commands.spawn((Observer::new(on_new_peer_connection), ExplorationObserver));
    commands.spawn((Observer::new(on_peer_deconection), ExplorationObserver));
    commands.spawn((Observer::new(on_duel_demand_received), ExplorationObserver));
    commands.spawn((Observer::new(on_duel_refused), ExplorationObserver));
    commands.spawn((Observer::new(on_duel_accepted), ExplorationObserver));
    commands.spawn((Observer::new(on_duel_cancelled), ExplorationObserver));
    commands.spawn((
        Observer::new(
            |trigger: Trigger<ui::popup::ButtonEvent>,
             mut next_gamemode: ResMut<NextState<GameMode>>,
             mut commands: Commands,
             sender_network_id: Res<SenderNetworkId>| match trigger.event() {
                ui::popup::ButtonEvent::Primary => {
                    commands.trigger(socket::SendMessageEvent::ToPeer(
                        socket::Message::DuelAccepted,
                        sender_network_id.0,
                    ));
                    next_gamemode.set(GameMode::Combat);
                    commands.insert_resource(combat::Owner(sender_network_id.0));
                }
                ui::popup::ButtonEvent::Secondary => {
                    commands.trigger(socket::SendMessageEvent::ToPeer(
                        socket::Message::DuelRefused,
                        sender_network_id.0,
                    ));
                }
            },
        ),
        ExplorationObserver,
    ));
    socket::start_connection(commands);
}

#[derive(Resource)]
struct SenderNetworkId(PeerId);

fn on_duel_demand_received(
    trigger: Trigger<socket::DuelDemandReceivedEvent>,
    mut commands: Commands,
    q_player: Query<(&Player, &socket::Id)>,
) {
    info!("Duel demand received");
    let player = q_player.iter().find(|(_, id)| id.0 == trigger.0);
    if let Some((player, _id)) = player {
        let sender_network_id = trigger.0.clone();
        commands.insert_resource(SenderNetworkId(sender_network_id));
        ui::popup::spawn_with_choices(
            &mut commands,
            format!("{} want to fight!", player.name),
            "Accept",
            "Decline",
        )
        .observe(
            |trigger: Trigger<ui::popup::ButtonEvent>, mut commands: Commands| {
                todo!("Make it work");
            },
        );
    }
}

fn on_duel_accepted(
    _trigger: Trigger<socket::DuelAcceptedEvent>,
    mut next_gamemode: ResMut<NextState<GameMode>>,
    mut commands: Commands,
    q_popup: Query<Entity, With<ui::popup::Popup>>,
    my_id: Res<socket::MyId>,
) {
    info!("Duel accepted");
    for popup in q_popup.iter() {
        commands.entity(popup).despawn_recursive();
    }
    next_gamemode.set(GameMode::Combat);
    commands.insert_resource(combat::Owner(my_id.0));
}

fn on_duel_refused(
    _trigger: Trigger<socket::DuelRefusedEvent>,
    mut commands: Commands,
    q_popup: Query<Entity, With<ui::popup::Popup>>,
) {
    info!("Duel refused");
    for popup in q_popup.iter() {
        commands.entity(popup).despawn_recursive();
    }
}

fn on_new_peer_connection(
    trigger: Trigger<socket::NewPeerConnectionEvent>,
    mut commands: Commands,
    q_transform: Query<&Transform, With<MainPlayer>>,
    player_info: Res<player::Info>,
    my_network_id: Res<socket::MyId>,
) {
    info!("New peer connected: {:?}", trigger.0);
    let transform = q_transform.single();
    let message = socket::Message::PlayerInitInfo {
        id: my_network_id.0,
        name: player_info.name.clone(),
        x: transform.translation.x,
        z: transform.translation.z,
    };
    commands.trigger(socket::SendMessageEvent::ToPeer(message, trigger.0));
}

fn on_peer_deconection(
    trigger: Trigger<socket::PeerDeconectionEvent>,
    mut commands: Commands,
    q_players: Query<(Entity, &socket::Id), With<Player>>,
) {
    info!("Peer disconnected: {:?}", trigger.0);
    for player in q_players.iter() {
        if player.1 .0 == trigger.0 {
            commands.entity(player.0).despawn_recursive();
            break;
        }
    }
}

fn on_peer_position_update(
    trigger: Trigger<socket::UpdatePlayerPositionEvent>,
    mut commands: Commands,
    q_players: Query<(Entity, &socket::Id), With<Player>>,
) {
    for player in q_players.iter() {
        if player.1 .0 == trigger.1 {
            commands
                .entity(player.0)
                .insert(Transform::from_translation(Vec3::new(
                    trigger.0.x,
                    player::PLAYER_SIZES.y / 2.0,
                    trigger.0.z,
                )));
            break;
        }
    }
}

fn on_duel_cancelled(
    _trigger: Trigger<socket::DuelCancelEvent>,
    mut commands: Commands,
    q_popup: Query<Entity, With<ui::popup::Popup>>,
) {
    info!("Duel cancelled");
    for popup in q_popup.iter() {
        commands.entity(popup).despawn_recursive();
    }
}

fn replicate_player_position(
    q_players: Query<&Transform, With<MainPlayer>>,
    mut commands: Commands,
) {
    let main_player = q_players.single();
    let translation = main_player.translation;
    commands.trigger(socket::SendMessageEvent::Broadcast(
        socket::Message::UpdatePlayerPosition(socket::UpdatePlayerPosition {
            x: translation.x,
            z: translation.z,
        }),
    ));
}
