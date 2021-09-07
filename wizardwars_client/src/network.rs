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
        network_channels_setup, ClientMessage, LobbyClientMessage, LobbyServerMessage,
        ServerMessage,
    },
};

use crate::arena::InsertPlayerEvent;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(NetworkingPlugin::default())
            .add_startup_system(network_channels_setup.system())
            .add_startup_system(client_setup_system.system())
            .add_system(handle_network_events_system.system())
            .add_system(read_server_message_channel_system.system());
    }
}

pub fn read_component_channel_system<C: ChannelMessage>(
    mut cmd: Commands,
    mut net: ResMut<NetworkResource>,
    players_query: Query<(&NetworkId, Entity)>,
) {
    let players: HashMap<&NetworkId, Entity> = players_query.iter().map(|(b, e)| (b, e)).collect();

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
    mut net: ResMut<NetworkResource>,
    mut events: EventWriter<InsertPlayerEvent>,
) {
    // TODO: remove this
    let mut is_loaded = false;

    for (_, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(message) = channels.recv::<ServerMessage>() {
            match message {
                ServerMessage::Lobby(msg) => match msg {
                    LobbyServerMessage::Welcome(id) => {
                        info!("Welcome message received: {:?}", id);
                    }
                    LobbyServerMessage::SetHost(id) => {
                        info!("Host now is: {:?}", id);
                    }
                    LobbyServerMessage::PlayerJoined(name) => {
                        info!("Player joined lobby: {}", name);
                    }
                    LobbyServerMessage::ReadyState(ready) => {
                        info!("Server Ready state changed: {:?}", ready);
                    }
                    LobbyServerMessage::StartLoading => {
                        info!("Start loading");
                        is_loaded = true;
                    }
                    LobbyServerMessage::PlayersList(_) => todo!(),
                },
                ServerMessage::Loading(e) => {
                    info!("Loading event received {:?}", e);
                }
                ServerMessage::Shopping(e) => {
                    info!("Shopping event received {:?}", e);
                }
                ServerMessage::InsertLocalPlayer(id, position) => {
                    events.send(InsertPlayerEvent {
                        id,
                        position,
                        is_local: true,
                    });
                }
                ServerMessage::InsertPlayer(id, position) => {
                    events.send(InsertPlayerEvent {
                        id,
                        position,
                        is_local: false,
                    });
                }
            }
        }
    }
    if is_loaded {
        net.broadcast_message(ClientMessage::Loaded);
    }
}
