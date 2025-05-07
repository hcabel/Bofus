use bevy::{math::u8, prelude::*};
use bevy_matchbox::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    materials::player_shadow::PlayerShadowMaterial,
    player::{self, spawn_player_character},
};

pub const SS_ADDRESS: &str = "ws://localhost:3536";

pub fn init(app: &mut App) {
    app.add_systems(
        Update,
        (
            poll_messages.run_if(is_connected),
            poll_id
                .run_if(is_connected)
                .run_if(not(resource_exists::<MyId>)),
        ),
    )
    .add_observer(send_queued_messages);
}

pub fn start_connection(mut commands: Commands) {
    let socket = MatchboxSocket::new_reliable(SS_ADDRESS);
    info!("Socket created at: {}", SS_ADDRESS);
    commands.insert_resource(socket);
}

#[derive(Debug, Resource)]
pub struct MyId(pub PeerId);

fn poll_id(mut commands: Commands, mut socket: ResMut<MatchboxSocket<SingleChannel>>) {
    if let Some(id) = socket.id() {
        commands.insert_resource(MyId(id));
    }
}

pub fn is_connected(socket: Option<Res<MatchboxSocket<SingleChannel>>>) -> bool {
    if let Some(socket) = socket {
        !socket.is_closed()
    } else {
        false
    }
}

#[derive(Component)]
pub struct Id(pub PeerId);

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    PlayerInitInfo {
        id: PeerId,
        name: String,
        x: f32,
        z: f32,
    },
    UpdatePlayerPosition(UpdatePlayerPosition),
    DuelDemand,
    DuelAccepted,
    DuelRefused,
    DuelCancelled,
    CombatPlayerJoined {
        stats: player::Info,
        position: Vec3,
    },
    CombatStart,
    CombatReadyStateChanged(bool),
}

#[derive(Debug, Event)]
pub struct NewPeerConnectionEvent(pub PeerId);

#[derive(Debug, Event)]
#[allow(dead_code)]
pub struct PeerDeconectionEvent(pub PeerId);

#[derive(Debug, Event)]
pub struct CombatReadyStateChangedEvent {
    pub peer_id: PeerId,
    pub is_ready: bool,
}

#[derive(Debug, Event)]
pub struct CombatStartedEvent;

#[derive(Debug, Event)]
pub struct CombatPlayerJoinedEvent {
    pub stats: player::Info,
    pub position: Vec3,
    pub peer_id: PeerId,
}

#[derive(Debug, Event)]
pub struct DuelDemandReceivedEvent(pub PeerId);

#[derive(Debug, Event)]
pub struct DuelRefusedEvent;

#[derive(Debug, Event)]
pub struct DuelAcceptedEvent;

#[derive(Debug, Event)]
pub struct DuelCancelEvent;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePlayerPosition {
    pub x: f32,
    pub z: f32,
}

#[derive(Debug, Event)]
pub struct UpdatePlayerPositionEvent(pub UpdatePlayerPosition, pub PeerId);

pub fn poll_messages(
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_shadow_materials: ResMut<Assets<PlayerShadowMaterial>>,
) {
    let peer_changes = socket.update_peers();
    for peer_change in peer_changes {
        match peer_change.1 {
            PeerState::Connected => commands.trigger(NewPeerConnectionEvent(peer_change.0)),
            PeerState::Disconnected => commands.trigger(PeerDeconectionEvent(peer_change.0)),
        }
    }

    for received in socket.receive() {
        let message = bincode::deserialize::<Message>(&*received.1).unwrap();
        match message {
            Message::PlayerInitInfo { id, name, x, z } => {
                spawn_player_character(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut player_shadow_materials,
                    crate::player::Player { name: name.clone() },
                    Vec3::new(x, player::PLAYER_SIZES.y / 2.0, z),
                )
                .insert(Id(id));
            }
            Message::UpdatePlayerPosition(new_position) => {
                commands.trigger(UpdatePlayerPositionEvent(new_position, received.0));
            }
            Message::DuelDemand => {
                commands.trigger(DuelDemandReceivedEvent(received.0));
            }
            Message::DuelRefused => {
                commands.trigger(DuelRefusedEvent);
            }
            Message::DuelAccepted => {
                commands.trigger(DuelAcceptedEvent);
            }
            Message::DuelCancelled => {
                commands.trigger(DuelCancelEvent);
            }
            Message::CombatReadyStateChanged(is_ready) => {
                commands.trigger(CombatReadyStateChangedEvent {
                    peer_id: received.0,
                    is_ready,
                });
            }
            Message::CombatStart => {
                commands.trigger(CombatStartedEvent);
            }
            Message::CombatPlayerJoined { stats, position } => {
                commands.trigger(CombatPlayerJoinedEvent {
                    stats,
                    position,
                    peer_id: received.0,
                });
            }
            _ => {
                warn!("Received unknown message: {:?}", message);
            }
        }
    }
}

#[derive(Debug, Event)]
pub enum SendMessageEvent {
    ToPeer(Message, PeerId),
    Broadcast(Message),
}

pub fn send_queued_messages(
    trigger: Trigger<SendMessageEvent>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
) {
    let event: &SendMessageEvent = trigger.event();
    match event {
        SendMessageEvent::ToPeer(message, peer_id) => {
            let message = bincode::serialize(&message).unwrap().into_boxed_slice();
            socket.send(message, *peer_id);
        }
        SendMessageEvent::Broadcast(message) => {
            let message = bincode::serialize(&message).unwrap().into_boxed_slice();
            let peers = socket.connected_peers().collect::<Vec<_>>();
            for peer in peers.iter() {
                socket.send(message.clone(), *peer);
            }
        }
    };
}
