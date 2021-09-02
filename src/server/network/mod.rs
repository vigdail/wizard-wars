use crate::common::components::{Client, NetworkId, Position};
use crate::common::messages::{
    network_channels_setup, ActionMessage, ClientMessage, LobbyClientMessage, ServerMessage,
};
use crate::common::network::{Dest, Pack};
use crate::server::lobby::{LobbyEvent, LobbyEventEntry};
use crate::server::states::ServerState;
use crate::server::InputEvent;
use bevy::prelude::*;
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Default)]
pub struct CurrentId(pub u32);

#[derive(Default)]
pub struct Host(pub Option<NetworkId>);

pub struct NetworkPlugin;

pub type ServerPacket = Pack<ServerMessage>;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(CurrentId::default())
            .insert_resource(Host::default())
            .add_event::<ServerPacket>()
            .add_plugin(NetworkingPlugin::default())
            .add_system_set(
                SystemSet::on_enter(ServerState::Init)
                    .with_system(network_channels_setup.system())
                    .with_system(server_setup_system.system()),
            )
            .add_system(handle_network_events_system.system())
            .add_system(read_network_channels_system.system())
            .add_system(send_packets_system.system())
            .add_system(broadcast_changes_system.system());
    }
}

pub fn server_setup_system(mut net: ResMut<NetworkResource>) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(ip_address, 9001);
    net.listen(socket_address, None, None);
    println!("Listening...");
}

fn handle_network_events_system(
    mut net: ResMut<NetworkResource>,
    mut network_event_reader: EventReader<NetworkEvent>,
) {
    for event in network_event_reader.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(_connection) => {
                    println!("New connection handle: {:?}", &handle);
                }
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            NetworkEvent::Disconnected(handle) => {
                println!("Disconnected handle: {:?}", &handle);
            }
            _ => {}
        }
    }
}

fn read_network_channels_system(
    mut net: ResMut<NetworkResource>,
    mut input_events: EventWriter<InputEvent>,
    mut lobby_events: EventWriter<LobbyEvent>,
) {
    for (handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        let client = Client(*handle);
        while let Some(message) = channels.recv::<ClientMessage>() {
            match message {
                ClientMessage::LobbyMessage(msg) => match msg {
                    LobbyClientMessage::Join(name) => lobby_events
                        .send(LobbyEvent::new(client, LobbyEventEntry::ClientJoined(name))),
                    LobbyClientMessage::ChangeReadyState(ready) => lobby_events.send(
                        LobbyEvent::new(client, LobbyEventEntry::ReadyChanged(ready)),
                    ),
                    LobbyClientMessage::GetPlayerList => todo!(),
                    LobbyClientMessage::StartGame => {
                        lobby_events.send(LobbyEvent::new(client, LobbyEventEntry::StartGame));
                    }
                },
                ClientMessage::Action(e) => match e {
                    ActionMessage::Move(dir) => {
                        input_events.send(InputEvent::Move(Client(*handle), dir));
                    }
                },
                ClientMessage::Loaded => {}
            }
        }
    }
}

fn send_packets_system(
    mut net: ResMut<NetworkResource>,
    mut events: EventReader<ServerPacket>,
    clients: Query<&Client>,
) {
    for pack in events.iter() {
        match &pack.dest {
            Dest::Single(client) => {
                net.send_message(client.0, pack.msg.clone())
                    .expect("Unable to send message");
            }
            Dest::AllExcept(exclude_client) => {
                for &client in clients.iter() {
                    if exclude_client != &client {
                        net.send_message(client.0, pack.msg.clone())
                            .expect("Unable to send message");
                    }
                }
            }
            Dest::All => {
                for client in clients.iter() {
                    net.send_message(client.0, pack.msg.clone())
                        .expect("Unable to send message");
                }
            }
        }
    }
}

fn broadcast_changes_system(
    mut net: ResMut<NetworkResource>,
    changed_positions: Query<(&NetworkId, &Position), Changed<Position>>,
) {
    for (id, position) in changed_positions.iter() {
        let _ = net.broadcast_message((*id, *position));
    }
}
