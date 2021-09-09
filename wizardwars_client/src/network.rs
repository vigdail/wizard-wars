use crate::{arena::InsertPlayerEvent, lobby::LobbyEvent};
use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use turbulence::message_channels::ChannelMessage;
use wizardwars_shared::{
    components::NetworkId,
    messages::{
        client_messages::{ClientMessage, LobbyClientMessage},
        network_channels_setup,
        server_messages::{LobbyServerMessage, ServerMessage},
    },
};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<ClientMessage>()
            .add_plugin(NetworkingPlugin::default())
            .add_startup_system(network_channels_setup.system())
            .add_startup_system(client_setup_system.system())
            .add_system(handle_network_events_system.system())
            .add_system(send_packets_system.system())
            .add_system(read_server_message_channel_system.system());
    }
}

pub fn read_component_channel_system<C: ChannelMessage>(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    players_query: Query<(&NetworkId, Entity)>,
) {
    let players: HashMap<&NetworkId, Entity> =
        players_query.iter().map(|(id, e)| (id, e)).collect();

    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();
        while let Some((network_id, component)) = channels.recv::<(NetworkId, C)>() {
            match players.get(&network_id) {
                Some(entity) => {
                    cmd.entity(*entity).insert(component);
                }
                None => {
                    warn!("No player found with id: {:?}", network_id);
                }
            }
        }
    }
}

fn client_setup_system(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(ip_address, 9001);
    info!("Connecting to {}...", socket_address);
    net.connect(socket_address);
}

fn handle_network_events_system(
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
) {
    for event in network_event_reader.iter() {
        if let NetworkEvent::Connected(handle) = event {
            match net.connections.get_mut(handle) {
                Some(_connection) => {
                    info!("Connection successful");

                    net.send_message(
                        *handle,
                        ClientMessage::LobbyMessage(LobbyClientMessage::Join(
                            "John Doe".to_owned(),
                        )),
                    )
                    .expect("Could not send hello");
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            }
        }
    }
}

fn read_server_message_channel_system(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    mut player_events: EventWriter<InsertPlayerEvent>,
    mut lobby_events: EventWriter<LobbyEvent>,
) {
    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ServerMessage>() {
            info!("Received message: {:?}", message);
            match message {
                ServerMessage::Lobby(msg) => match msg {
                    LobbyServerMessage::Welcome(id) => {
                        cmd.spawn().insert(id);
                    }
                    LobbyServerMessage::SetHost(_) => {}
                    LobbyServerMessage::PlayerJoined(id, _) => {
                        cmd.spawn().insert(id);
                    }
                    LobbyServerMessage::ReadyState(_) => {}
                    LobbyServerMessage::StartLoading => {
                        lobby_events.send(LobbyEvent::StartLoading);
                    }
                    LobbyServerMessage::PlayersList(_) => todo!(),
                },
                ServerMessage::Loading(_) => {}
                ServerMessage::Shopping(_) => {}
                ServerMessage::InsertLocalPlayer(id, position) => {
                    player_events.send(InsertPlayerEvent {
                        id,
                        position,
                        is_local: true,
                    });
                }
                ServerMessage::InsertPlayer(id, position) => {
                    player_events.send(InsertPlayerEvent {
                        id,
                        position,
                        is_local: false,
                    });
                }
            }
        }
    }
}

fn send_packets_system(mut net: ResMut<NetworkResource>, mut events: EventReader<ClientMessage>) {
    for message in events.iter() {
        net.broadcast_message(message.clone());
    }
}
